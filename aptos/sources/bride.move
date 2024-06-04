module bridge::aptos_nft_bridge {
  use std::signer;
  use std::vector;
  use std::hash;
  use std::string::{Self, String};
  use std::bcs;
  use std::option::{Self};
  use std::debug;
  use aptos_std::ed25519;
  use aptos_std::simple_map::{Self, SimpleMap};
  use aptos_framework::object;
  use aptos_framework::primary_fungible_store;
  use aptos_framework::fungible_asset::{Self, Metadata};
  use aptos_framework::coin;
  use aptos_framework::event;
  use aptos_framework::aptos_coin::AptosCoin;
  use aptos_framework::table::{Self, Table};
  use aptos_framework::account::{Self, SignerCapability};
  use aptos_token_objects::aptos_token::{AptosToken};
  use aptos_token_objects::token;
  use aptos_token_objects::royalty::{Self};
  use aptos_token_objects::collection;

  const E_ALREADY_INITIALIZED: u64 = 0;
  const E_NOT_BRIDGE_ADMIN: u64 = 1;
  const E_VALIDATORS_LENGTH_ZERO: u64 = 2;
  const E_VALIDATOR_ALREADY_EXIST: u64 = 3;
  const THERSHOLD_NOT_REACHED: u64 = 4;
  const E_INVALID_GK: u64 = 5;
  const E_INVALID_SIGNATURE: u64 = 6;
  const E_NOT_INITIALIZED: u64 = 7;
  const E_INVALID_FEE: u64 = 8;
  const E_NO_REWARDS_AVAILABLE: u64 = 9;
  const E_INVALID_DESTINATION_CHAIN: u64 = 9;
  const E_INVALID_NFT_TYPE: u64 = 10;

  const TYPE_ERC721: vector<u8> = b"singular";
  const TYPE_ERC1155: vector<u8> = b"multiple";

  struct Validator has drop, store, copy {
    pending_reward: u64
  }
  
  struct OriginalToDuplicateKey has drop, store, copy {
    source_chain: vector<u8>,
    source_contract: vector<u8>
  }

  struct OriginalToDuplicateInfo has drop, store, copy {
    self_chain: vector<u8>,
    collection_address: vector<u8>
  }
  
  struct DuplicateToOriginalInfo has drop, store, copy {
    source_chain: vector<u8>,
    source_contract: vector<u8>
  }

  struct DuplicateToOriginalKey has drop, store, copy {
    self_chain: vector<u8>,
    collection_address: vector<u8>
  }

  struct CollectionObject has drop, store, copy {
    collection: address,
    object: address,
  }

  struct CollectionNftObject has drop, store, copy {
    collection: address,
    nft_address: address
  }

  struct Bridge has key {
    validators: SimpleMap<vector<u8>, Validator>,
    signer_cap: SignerCapability,
    collection_objects: Table<CollectionObject, u256>,
    nfts_counter: u64,
    original_to_duplicate_mapping: Table<OriginalToDuplicateKey, OriginalToDuplicateInfo>,
    duplicate_to_original_mapping: Table<DuplicateToOriginalKey, DuplicateToOriginalInfo>,
    nft_collection_tokens: Table<CollectionNftObject, u256>,
    nft_collections_counter: SimpleMap<address, u256>,
    self_chain: vector<u8>
  }

  struct SignatureInfo has drop {
    public_key: vector<u8>,
    signature: vector<u8>,
  }

  struct CalimData has drop, copy {
    user: address,
    token_id: u256,
    collection: String,
    name: String,
    description: String,
    uri: String,
    royalty_points_numerator: u64,
    royalty_points_denominator: u64,
    royalty_payee_address: address,
    fee: u64,
    source_chain: vector<u8>,
    source_nft_contract_address: vector<u8>,
    destination_chain: vector<u8>,
    transaction_hash: vector<u8>,
    nft_type: vector<u8>
  }

  #[event]
  struct AddNewValidatorEvent has drop, store {
    validator: vector<u8>,
  }

  #[event]
  struct RewardValidatorEvent has drop, store {
    validator: vector<u8>,
  }

  #[event]
  struct LockedEvent has drop, store {
    token_id: u256,
    user_address: address,
    token_amount: u64,
    nft_type: vector<u8>,
    destination_chain: vector<u8>,
    self_chain: vector<u8>,
    source_nft_contract_address: vector<u8>
  }

  #[event]
  struct UnLock721Event has drop, store {
    to: address,
    token_id: u256,
  }

  #[event]
  struct UnLock1155Event has drop, store {
    to: address,
    token_id: u256,
    amount: u64
  }

  #[event]
  struct Claim721Event has drop, store {
    source_chain: vector<u8>,
    token_id: u256,
    transaction_hash: vector<u8>,
    nft_contract: vector<u8>
  }

  #[event]
  struct Claim1155Event has drop, store {
    source_chain: vector<u8>,
    token_id: u256,
    amount: u64,
    transaction_hash: vector<u8>,
    nft_contract: vector<u8>
  }
  
  fun is_sig_valid(
    signature: vector<u8>,
    public_key: vector<u8>,
    data: vector<u8>,
  ): bool {
    let pk = ed25519::new_unvalidated_public_key_from_bytes(public_key);
    let sig = ed25519::new_signature_from_bytes(signature);
    let verified = ed25519::signature_verify_strict(&sig, &pk, hash::sha2_256(data));
    verified
  }

  fun assert_meets_validator_thershold(
    signatures: vector<vector<u8>>, 
    public_keys: &vector<vector<u8>>, 
    data: vector<u8>, 
    validators: &SimpleMap<vector<u8>, Validator>
  ) {
    let signatures_count = vector::length(&signatures);

    let percentage = &mut 0;
    let processed_validators = &mut vector::empty<vector<u8>>(); 

    for (i in 0..(signatures_count - 1)) {
      let signature = *vector::borrow(&signatures, i);
      let public_key = *vector::borrow(public_keys, i);
      let valid_signature = is_sig_valid(signature, public_key, bcs::to_bytes(&data));
      let is_validator = simple_map::contains_key(validators, &public_key);
      let validator_already_processed = vector::contains(processed_validators, &public_key);

      if(valid_signature && is_validator && !validator_already_processed) {
        vector::push_back(processed_validators, public_key);
        *percentage = *percentage + 1;
      }
    };

    let validators_count = vector::length(processed_validators);
    assert!(*percentage >= ((validators_count * 2) / 3) + 1, THERSHOLD_NOT_REACHED);
  }

  fun retrieve_bridge_resource_address(): address acquires Bridge {
    let bridge_data = borrow_global<Bridge>(@bridge);
    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    signer::address_of(&bridge_signer_from_cap)
  }

  public entry fun initialize(
    admin: &signer, 
    validators: vector<vector<u8>>, 
    seed: vector<u8>, 
    self_chain: vector<u8>
  ) {
    let admin_addr = signer::address_of(admin);
    assert!(admin_addr == @bridge, E_NOT_BRIDGE_ADMIN);
    let total_validators = vector::length(&validators);
    assert!(!exists<Bridge>(admin_addr), E_ALREADY_INITIALIZED);
    assert!(total_validators > 0, E_VALIDATORS_LENGTH_ZERO);

    let (_, signer_cap) = account::create_resource_account(admin, seed);
    let validators_to_add = &mut simple_map::create<vector<u8>, Validator>();

    for (i in 0..(total_validators - 1)) {
      let validator = *vector::borrow(&validators, i);
      simple_map::add(validators_to_add, validator, Validator { pending_reward: 0 });
    };

    move_to<Bridge>(
      admin,
      Bridge {
        self_chain,
        validators: *validators_to_add,
        signer_cap,
        collection_objects: table::new(),
        nfts_counter: 0,
        original_to_duplicate_mapping: table::new(),
        duplicate_to_original_mapping: table::new(),
        nft_collection_tokens: table::new(),
        nft_collections_counter: simple_map::create<address, u256>()
      }
    );
  }

  public entry fun add_validator(
    validator: vector<u8>, 
    signatures: vector<vector<u8>>, 
    public_keys: vector<vector<u8>>
  ) acquires Bridge {
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    let signatures_count = vector::length(&signatures);

    assert!(signatures_count > 0, E_VALIDATORS_LENGTH_ZERO);
    assert!(!simple_map::contains_key(&mut bridge_data.validators, &validator), E_VALIDATOR_ALREADY_EXIST);

    assert_meets_validator_thershold(signatures, &public_keys, bcs::to_bytes(&validator), &bridge_data.validators);

    event::emit(AddNewValidatorEvent { validator });
    simple_map::add(&mut bridge_data.validators, validator, Validator { pending_reward: 0 });
  }

  public entry fun claim_validator_rewards(
    admin: signer,
    to: address,
    validator: vector<u8>, 
    signatures: vector<vector<u8>>, 
    public_keys: vector<vector<u8>>
  ) acquires Bridge {
    
    let admin_addr = signer::address_of(&admin);
    assert!(admin_addr == @bridge, E_NOT_BRIDGE_ADMIN);
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

    let bridge_data = borrow_global_mut<Bridge>(@bridge);

    assert_meets_validator_thershold(signatures, &public_keys, bcs::to_bytes(&validator), &bridge_data.validators);


    let validator_reward = simple_map::borrow_mut(&mut bridge_data.validators, &validator);
    *validator_reward = Validator { pending_reward: 0 };

    event::emit(RewardValidatorEvent { validator });
    coin::transfer<AptosCoin>(&admin, to, validator_reward.pending_reward);
  }

  public entry fun lock_721(
    owner: &signer, 
    collection: String, 
    name: String, 
    destination_chain: vector<u8>, 
    _token_id: u256,
    source_nft_contract_address: vector<u8>,
  ) acquires Bridge {
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

    let owner_address = signer::address_of(owner);

    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);

    let token_addr = token::create_token_address(&owner_address, &collection, &name);
    let token_object = object::address_to_object<AptosToken>(token_addr);
    
    let collection_address = collection::create_collection_address(&bridge_resource_addr, &collection);
    
    let nft_already_exists = table::contains(&mut bridge_data.nft_collection_tokens, CollectionNftObject { collection: collection_address, nft_address: token_addr});
    
    let nft_token_id: &u256;
    
    if(nft_already_exists) {
      nft_token_id = table::borrow(&mut bridge_data.nft_collection_tokens, CollectionNftObject { collection: collection_address, nft_address: token_addr})
    } else {
      let collection_counter_intilaized = simple_map::contains_key(&bridge_data.nft_collections_counter, &collection_address);
      if (collection_counter_intilaized) {
        let nft_collection_counter = simple_map::borrow_mut(&mut bridge_data.nft_collections_counter, &collection_address);
        *nft_collection_counter = *nft_collection_counter + 1;
        nft_token_id = &(*nft_collection_counter + 1);
      } else {
        simple_map::add(&mut bridge_data.nft_collections_counter, collection_address, 0);
        nft_token_id = &0;
      };
      table::add(&mut bridge_data.nft_collection_tokens, CollectionNftObject { collection: collection_address, nft_address: token_addr}, *nft_token_id);
    };

    let key_duplicate = DuplicateToOriginalKey {
      self_chain: bridge_data.self_chain,
      collection_address: bcs::to_bytes(&source_nft_contract_address),
    };
    
    let is_original = !table::contains(&mut bridge_data.duplicate_to_original_mapping, key_duplicate);

    if(is_original) {
      event::emit(LockedEvent {
        token_id: *nft_token_id, 
        user_address: owner_address, 
        token_amount: 1, 
        nft_type: TYPE_ERC721, 
        destination_chain,
        self_chain: bridge_data.self_chain,
        source_nft_contract_address
      });
    } else {
      let original_collection_address = table::borrow(&mut bridge_data.duplicate_to_original_mapping, key_duplicate);

      event::emit(LockedEvent { 
        token_id: *nft_token_id, 
        user_address: owner_address, 
        token_amount: 1, 
        nft_type: TYPE_ERC721, 
        destination_chain,
        self_chain: original_collection_address.source_chain,
        source_nft_contract_address: original_collection_address.source_contract
      });
    };

    object::transfer(owner, token_object, bridge_resource_addr);
  }

  public entry fun lock_1155(
    owner: &signer, 
    collection: String, 
    name: String, 
    amount: u64, 
    destination_chain: vector<u8>, 
    _token_id: u256,
    source_nft_contract_address: vector<u8>,
  ) acquires Bridge {
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);
    
    let owner_address = signer::address_of(owner);

    let token_addr = token::create_token_address(&owner_address, &collection, &name);
    let token_object = object::address_to_object<AptosToken>(token_addr);

    let collection_address = collection::create_collection_address(&bridge_resource_addr, &collection);
    
    let nft_already_exists = table::contains(&mut bridge_data.nft_collection_tokens, CollectionNftObject { collection: collection_address, nft_address: token_addr});
    
    let nft_token_id: &u256;
    
    if(nft_already_exists) {
      nft_token_id = table::borrow(&mut bridge_data.nft_collection_tokens, CollectionNftObject { collection: collection_address, nft_address: token_addr})
    } else {
      let collection_counter_intilaized = simple_map::contains_key(&bridge_data.nft_collections_counter, &collection_address);
      if (collection_counter_intilaized) {
        let nft_collection_counter = simple_map::borrow_mut(&mut bridge_data.nft_collections_counter, &collection_address);
        *nft_collection_counter = *nft_collection_counter + 1;
        nft_token_id = &(*nft_collection_counter + 1);
      } else {
        simple_map::add(&mut bridge_data.nft_collections_counter, collection_address, 0);
        nft_token_id = &0;
      };
      table::add(&mut bridge_data.nft_collection_tokens, CollectionNftObject { collection: collection_address, nft_address: token_addr}, *nft_token_id);
    };

    let key_duplicate = DuplicateToOriginalKey {
      self_chain: bridge_data.self_chain,
      collection_address: bcs::to_bytes(&source_nft_contract_address),
    };
    
    let is_original = !table::contains(&mut bridge_data.duplicate_to_original_mapping, key_duplicate);

    if(is_original) {
      event::emit(LockedEvent { 
        token_id: *nft_token_id, 
        user_address: owner_address, 
        token_amount: amount, 
        nft_type: TYPE_ERC1155, 
        destination_chain,
        self_chain: bridge_data.self_chain,
        source_nft_contract_address
      });
    } else {
      let original_collection_address = table::borrow(&mut bridge_data.duplicate_to_original_mapping, key_duplicate);

      event::emit(LockedEvent { 
        token_id: *nft_token_id, 
        user_address: owner_address, 
        token_amount: amount, 
        nft_type: TYPE_ERC1155, 
        destination_chain,
        self_chain: original_collection_address.source_chain,
        source_nft_contract_address: original_collection_address.source_contract
      });
    };

    primary_fungible_store::transfer(
      owner,
      token_object,
      bridge_resource_addr,
      amount
    );
  }

  public entry fun claim_721(
    user: signer,
    collection: String,
    name: String,
    description: String,
    uri: String,
    royalty_points_numerator: u64,
    royalty_points_denominator: u64,
    royalty_payee_address: address,
    fee: u64,
    signatures: vector<vector<u8>>, 
    public_keys: vector<vector<u8>>,
    destination_chain: vector<u8>,
    source_chain: vector<u8>,
    source_nft_contract_address: vector<u8>,
    token_id: u256,
    transaction_hash: vector<u8>,
    nft_type: vector<u8>
  ) acquires Bridge {
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
    assert!(hash::sha2_256(nft_type) == hash::sha2_256(TYPE_ERC721), E_INVALID_NFT_TYPE);

    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    assert!(hash::sha2_256(bridge_data.self_chain) == hash::sha2_256(destination_chain), E_INVALID_DESTINATION_CHAIN);

    let user_addr = signer::address_of(&user);

    let data = CalimData {
      user: user_addr,
      token_id,
      collection,
      name,
      description,
      uri,
      royalty_points_numerator,
      royalty_points_denominator,
      royalty_payee_address,
      fee,
      destination_chain,
      source_chain,
      source_nft_contract_address,
      transaction_hash,
      nft_type
    };
    
    assert_meets_validator_thershold(signatures, &public_keys, bcs::to_bytes(&data), &bridge_data.validators);
    
    coin::transfer<AptosCoin>(&user, @bridge, fee);

    reward_validators(fee, &public_keys, &mut bridge_data.validators);

    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);
    
    let token_addr = token::create_token_address(&bridge_resource_addr, &collection, &name);
    let token_object = object::address_to_object<AptosToken>(token_addr);

    // if locked nft. transfer from resource account to user
    // else mint new one.
    let collection_address = collection::create_collection_address(&bridge_resource_addr, &collection);

    if (object::owner(token_object) == bridge_resource_addr) {
      event::emit(UnLock721Event { to: user_addr, token_id });

      object::transfer(&bridge_signer_from_cap, token_object, user_addr);
      
    } else {
      collection::create_unlimited_collection(
        &bridge_signer_from_cap,
        string::utf8(b""),
        collection,
        option::none(),
        string::utf8(b""),
      );

      let royalty = royalty::create(royalty_points_numerator, royalty_points_denominator, royalty_payee_address);

      token::create_named_token(
        &user,
        collection,
        description,
        name,
        option::some(royalty),
        uri,
      );

      let key = OriginalToDuplicateKey {
        source_chain,
        source_contract: bcs::to_bytes(&source_nft_contract_address),
      };

      let is_duplicate = !table::contains(&mut bridge_data.original_to_duplicate_mapping, key);

      if(is_duplicate) {

        let value = OriginalToDuplicateInfo {
          self_chain: bridge_data.self_chain,
          collection_address: bcs::to_bytes(&collection_address),
        };

        let key_duplicate = DuplicateToOriginalKey {
          self_chain: bridge_data.self_chain,
          collection_address: bcs::to_bytes(&collection_address),
        };

        let value_duplicate = DuplicateToOriginalInfo {
          source_chain,
          source_contract: bcs::to_bytes(&source_nft_contract_address),
        };

        table::add(&mut bridge_data.original_to_duplicate_mapping, key, value);

        table::add(&mut bridge_data.duplicate_to_original_mapping, key_duplicate, value_duplicate);
      }

    };

    event::emit(Claim721Event { 
      token_id, 
      transaction_hash, 
      source_chain, 
      nft_contract: bcs::to_bytes(&collection_address)
    });

  }

  public entry fun claim_1155(
    user: signer,
    collection: String,
    name: String,
    description: String,
    symbol: String,
    amount: u64,
    uri: String,
    icon_uri: String,
    project_uri: String,
    royalty_points_numerator: u64,
    royalty_points_denominator: u64,
    royalty_payee_address: address,
    fee: u64,
    signatures: vector<vector<u8>>, 
    public_keys: vector<vector<u8>>,
    source_chain: vector<u8>,
    source_nft_contract_address: vector<u8>,
    destination_chain: vector<u8>,
    transaction_hash: vector<u8>,
    token_id: u256,
    nft_type: vector<u8>
  ) acquires Bridge {
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

    assert!(hash::sha2_256(nft_type) == hash::sha2_256(TYPE_ERC1155), E_INVALID_NFT_TYPE);

    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    assert!(hash::sha2_256(bridge_data.self_chain) == hash::sha2_256(destination_chain), E_INVALID_DESTINATION_CHAIN);

    let user_addr = signer::address_of(&user);

    let data = CalimData {
      user: user_addr,
      token_id,
      collection,
      name,
      description,
      uri,
      royalty_points_numerator,
      royalty_points_denominator,
      royalty_payee_address,
      fee,
      source_chain,
      source_nft_contract_address,
      destination_chain,
      transaction_hash,
      nft_type
    };
    
    coin::transfer<AptosCoin>(&user, @bridge, fee);

    assert_meets_validator_thershold(signatures, &public_keys, bcs::to_bytes(&data), &bridge_data.validators);
    reward_validators(fee, &public_keys, &mut bridge_data.validators);

    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);
    
    let token_addr = token::create_token_address(&bridge_resource_addr, &collection, &name);
    let token_object = object::address_to_object<AptosToken>(token_addr);

    collection::create_unlimited_collection(
      &bridge_signer_from_cap,
      string::utf8(b""),
      collection,
      option::none(),
      string::utf8(b""),
    );
    let collection_address = collection::create_collection_address(&bridge_resource_addr, &collection);
    // if locked nft. transfer from resource account to user
    // else mint new one.
    if (object::owner(token_object) == bridge_resource_addr) {
      // unlock from resource account
      let metadata = object::convert<AptosToken, Metadata>(token_object);
      let store = primary_fungible_store::ensure_primary_store_exists(bridge_resource_addr, metadata);
      let balance_of_tokens = fungible_asset::balance(store);
      
      if (balance_of_tokens >= amount) {
        
        primary_fungible_store::transfer(
          &bridge_signer_from_cap,
          token_object,
          user_addr,
          amount
        );

        event::emit(UnLock1155Event { to: user_addr, token_id, amount });

      } else {
       
        let to_mint = amount - balance_of_tokens;
        
        primary_fungible_store::transfer(
          &bridge_signer_from_cap,
          token_object,
          user_addr,
          balance_of_tokens
        );
        event::emit(UnLock1155Event { to: user_addr, token_id, amount: balance_of_tokens });


        let royalty = royalty::create(royalty_points_numerator, royalty_points_denominator, royalty_payee_address);

        let new_nft_constructor_ref = &token::create(
          &user,
          collection,
          description,
          name,
          option::some(royalty),
          uri,
        );
        // Make this nft token fungible so there can multiple instances of it.
        primary_fungible_store::create_primary_store_enabled_fungible_asset(
          new_nft_constructor_ref,
          option::some((to_mint as u128)),
          name,
          symbol,
          0, // NFT cannot be divided so decimals is 0,
          icon_uri,
          project_uri,
        );
      }
    } else {

      let royalty = royalty::create(royalty_points_numerator, royalty_points_denominator, royalty_payee_address);

      let new_nft_constructor_ref = &token::create(
        &user,
        collection,
        description,
        name,
        option::some(royalty),
        uri,
      );
      // Make this nft token fungible so there can multiple instances of it.
      primary_fungible_store::create_primary_store_enabled_fungible_asset(
        new_nft_constructor_ref,
        option::some((amount as u128)),
        name,
        symbol,
        0, // NFT cannot be divided so decimals is 0,
        icon_uri,
        project_uri,
      );
      let key = OriginalToDuplicateKey {
        source_chain,
        source_contract: bcs::to_bytes(&source_nft_contract_address),
      };

      let is_duplicate = table::contains(&mut bridge_data.original_to_duplicate_mapping, key);

      if(!is_duplicate) {

        let value = OriginalToDuplicateInfo {
          self_chain: bridge_data.self_chain,
          collection_address: bcs::to_bytes(&collection_address),
        };

        let key_duplicate = DuplicateToOriginalKey {
          self_chain: bridge_data.self_chain,
          collection_address: bcs::to_bytes(&collection_address),
        };

        let value_duplicate = DuplicateToOriginalInfo {
          source_chain,
          source_contract: bcs::to_bytes(&source_nft_contract_address),
        };

        table::add(&mut bridge_data.original_to_duplicate_mapping, key, value);

        table::add(&mut bridge_data.duplicate_to_original_mapping, key_duplicate, value_duplicate);
      }
    };

    event::emit(Claim1155Event {
      token_id, 
      transaction_hash, 
      source_chain, 
      amount, 
      nft_contract: bcs::to_bytes(&collection_address)
    });

  }

  fun reward_validators(
    fee: u64, 
    validators_to_reward: &vector<vector<u8>>, 
    all_validators: &mut SimpleMap<vector<u8>, Validator>
  ) {
    assert!(fee > 0, E_INVALID_FEE);

    let total_rewards = coin::balance<AptosCoin>(@bridge);
    assert!(total_rewards >= fee, E_NO_REWARDS_AVAILABLE);
    
    let total_validators_to_reward = vector::length(validators_to_reward);
    let fee_per_validator = fee / total_validators_to_reward;

    for (i in 0..(total_validators_to_reward - 1)) {
      let validator = vector::borrow(validators_to_reward, i);
      let validator_reward = simple_map::borrow_mut(all_validators, validator);
      *validator_reward = Validator { pending_reward: validator_reward.pending_reward + fee_per_validator};
    };

  }

  // #[test(admin = @bridge)]
  // // #[expected_failure(abort_code = E_ALREADY_INITIALIZED)]
  // public entry fun test_flow(admin: signer) acquires Bridge {

  //   let admin_addr = signer::address_of(&admin);
  //   // debug::print(&admin_addr);
    
  //   let validators: vector<vector<u8>> = vector::empty<vector<u8>>();
  //   vector::push_back(&mut validators, b"0x6");
  //   vector::push_back(&mut validators, b"0x7");

  //   // debug::print(&exists<Bridge>(admin_addr));
  //   initialize(&admin, validators, b"xyz", b"APTOS");

  //   lock_721(
  //     &admin, 
  //     string::utf8(b"Panda Collection"), 
  //     string::utf8(b"Panda # 01"), 
  //     b"BSC", 
  //     1,
  //     b"0x123",
  //   );

  //   claim_721(
  //     admin,
  //     string::utf8(b"Panda Collection"),
  //     string::utf8(b"Panda # 01"),
  //     string::utf8(b"First Panda Nft"),
  //     string::utf8(b"First Panda Nft"),
  //     10,
  //     2,
  //     admin_addr,
  //     10,
  //     vector<vector<u8>>[b"a",b"b"], 
  //     vector<vector<u8>>[b"a",b"b"],
  //     b"APTOS",
  //     b"BSC",
  //     b"0123",
  //     0,
  //     b"a123",
  //     b"singular"
  //   )

  //   let bridge_data = borrow_global<Bridge>(admin_addr);
  //   let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
  //   let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);

  //   let collection_address = collection::create_collection_address(&bridge_resource_addr, &string::utf8(b"Panda Collection"));
  //   let nft_collection_counter = simple_map::borrow(&bridge_data.nft_collections_counter, &collection_address);
  //   debug::print(&*nft_collection_counter);
  //   // assert!((vector::length(&bridge.validators) as u64) == 2, 1);
  //   // assert!(*vector::borrow(&bridge.validators, 0) == b"@0x6", 2);
  //   // assert!(*vector::borrow(&bridge.validators, 1) == b"@0x7", 3);
  //   // assert!(bridge.deployed_collections == vector::empty<address>(), 4);
  //   // debug::print(&address_to_vector(admin_addr));

  //   // initialize(&admin, validators);
  // }
}