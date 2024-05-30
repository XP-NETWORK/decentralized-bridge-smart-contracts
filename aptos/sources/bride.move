module bridge::aptos_nft_bridge {
  // TODO: add bridge intialzed check to all entry functions.

  use std::signer;
  use std::vector;
  use aptos_framework::object::{Self, Object};
  use aptos_token_objects::aptos_token::{AptosToken};
  use aptos_token_objects::token;
  use std::string::{String};
  use aptos_token_objects::royalty::{Self};
  use std::option::{Self};
  use aptos_framework::primary_fungible_store;
  use aptos_framework::fungible_asset;
  use aptos_std::ed25519;
  use std::hash;
  use std::bcs;
  use aptos_framework::coin;
  use aptos_framework::aptos_coin::AptosCoin;
  use aptos_std::simple_map::{Self, SimpleMap};
  use aptos_framework::account::{Self, SignerCapability};
  use std::debug;

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

  struct Validator has drop, store, copy {
    pending_reward: u64
  }

  struct Bridge has key {
    validators: SimpleMap<vector<u8>, Validator>,
    signer_cap: SignerCapability,
  }

  struct SignatureInfo has drop {
    public_key: vector<u8>,
    signature: vector<u8>,
  }

  struct CalimData has drop, copy {
    user: address,
    collection: String,
    name: String,
    description: String,
    uri: String,
    royalty_points_numerator: u64,
    royalty_points_denominator: u64,
    royalty_payee_address: address,
    fee: u64,
  }

  // TODO: initialize and emit events

  
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

  public entry fun initialize(admin: &signer, validators: vector<vector<u8>>, seed: vector<u8>) {
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
        validators: *validators_to_add,
        signer_cap
      }
    );
  }

  public entry fun add_validator(validator: vector<u8>, signatures: vector<vector<u8>>, public_keys: vector<vector<u8>>) acquires Bridge {
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    let signatures_count = vector::length(&signatures);

    assert!(signatures_count > 0, E_VALIDATORS_LENGTH_ZERO);
    assert!(!simple_map::contains_key(&mut bridge_data.validators, &validator), E_VALIDATOR_ALREADY_EXIST);

    should_meet_thershold(signatures, &public_keys, bcs::to_bytes(&validator), &bridge_data.validators);

    simple_map::add(&mut bridge_data.validators, validator, Validator { pending_reward: 0 });
  }

  fun should_meet_thershold(signatures: vector<vector<u8>>, public_keys: &vector<vector<u8>>, data: vector<u8>, validators: &SimpleMap<vector<u8>, Validator>) {
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

    should_meet_thershold(signatures, &public_keys, bcs::to_bytes(&validator), &bridge_data.validators);


    let validator_reward = simple_map::borrow_mut(&mut bridge_data.validators, &validator);
    *validator_reward = Validator { pending_reward: 0 };

    coin::transfer<AptosCoin>(&admin, to, validator_reward.pending_reward);
  }

  fun retrieve_bridge_resource_address(): address acquires Bridge {
    assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

    let bridge_data = borrow_global<Bridge>(@bridge);
    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    signer::address_of(&bridge_signer_from_cap)
  }

  public entry fun lock_721(owner: &signer, object: Object<AptosToken>) acquires Bridge {
    let bridge_resource_addr = retrieve_bridge_resource_address();

    object::transfer(owner, object, bridge_resource_addr);
  }

  public entry fun lock_1155(owner: &signer, object: Object<AptosToken>, amount: u64) acquires Bridge {
    let bridge_resource_addr = retrieve_bridge_resource_address();

    primary_fungible_store::transfer(
      owner,
      object,
      bridge_resource_addr,
      amount
    );
  }

  // TODO: Create Collection if donesn't exist already.
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
    public_keys: vector<vector<u8>>
  ) acquires Bridge {
    let user_addr = signer::address_of(&user);

    let data = CalimData {
      user: user_addr,
      collection,
      name,
      description,
      uri,
      royalty_points_numerator,
      royalty_points_denominator,
      royalty_payee_address,
      fee,
    };
    
    coin::transfer<AptosCoin>(&user, @bridge, fee);

    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    should_meet_thershold(signatures, &public_keys, bcs::to_bytes(&data), &bridge_data.validators);
    reward_validators(fee, &public_keys, &mut bridge_data.validators);

    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);
    
    let token_addr = token::create_token_address(&bridge_resource_addr, &collection, &name);
    let token_object = object::address_to_object<AptosToken>(token_addr);

    // if locked nft. transfer from resource account to user
    // else mint new one.
    if (object::owner(token_object) == bridge_resource_addr) {
      object::transfer(&bridge_signer_from_cap, token_object, user_addr);
    } else {
      let royalty = royalty::create(royalty_points_numerator, royalty_points_denominator, royalty_payee_address);
      token::create_named_token(
        &user,
        collection,
        description,
        name,
        option::some(royalty),
        uri,
      );
    }
  }

  // TODO: Create Collection if donesn't exist already.
  public entry fun claim_1155(
    user: signer,
    collection: String,
    name: String,
    description: String,
    symbol: String,
    maximum: u128,
    uri: String,
    icon_uri: String,
    project_uri: String,
    royalty_points_numerator: u64,
    royalty_points_denominator: u64,
    royalty_payee_address: address,
    fee: u64,
    signatures: vector<vector<u8>>, 
    public_keys: vector<vector<u8>>
  ) {

    let user_addr = signer::address_of(&user);

    let data = CalimData {
      user: user_addr,
      collection,
      name,
      description,
      uri,
      royalty_points_numerator,
      royalty_points_denominator,
      royalty_payee_address,
      fee,
    };
    
    coin::transfer<AptosCoin>(&user, @bridge, fee);

    let bridge_data = borrow_global_mut<Bridge>(@bridge);
    should_meet_thershold(signatures, &public_keys, bcs::to_bytes(&data), &bridge_data.validators);
    reward_validators(fee, &public_keys, &mut bridge_data.validators);

    let bridge_signer_from_cap = account::create_signer_with_capability(&bridge_data.signer_cap);
    let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);
    
    let token_addr = token::create_token_address(&bridge_resource_addr, &collection, &name);
    let token_object = object::address_to_object<AptosToken>(token_addr);

    // if locked nft. transfer from resource account to user
    // else mint new one.
    if (object::owner(token_object) == bridge_resource_addr) {
      let balance_of_tokens = fungible_asset::maximum(token_object);
      if(balance_of_tokens >= maximum) {
        primary_fungible_store::transfer(
          &bridge_signer_from_cap,
          object,
          user_addr,
          maximum
        );
      } else {
        let to_mint = maximum - balance_of_tokens;
        
        primary_fungible_store::transfer(
          &bridge_signer_from_cap,
          object,
          user_addr,
          balance_of_tokens
        );

        let royalty = royalty::create(royalty_points_numerator, royalty_points_denominator, royalty_payee_address);

        // TODO: donot create new ref instead store the ref and reuse it here.
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
          option::some(to_mint),
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
        option::some(maximum),
        name,
        symbol,
        0, // NFT cannot be divided so decimals is 0,
        icon_uri,
        project_uri,
      );
    }
  }

  fun reward_validators(fee: u64, validators_to_reward: &vector<vector<u8>>, all_validators: &mut SimpleMap<vector<u8>, Validator>) {
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
  // #[expected_failure(abort_code = E_ALREADY_INITIALIZED)]
  // public entry fun test_flow(admin: signer) acquires Bridge {
  //   use std::debug;

  //   let admin_addr = signer::address_of(&admin);
  //   debug::print(&admin_addr);
    
  //   let validators: vector<vector<u8>> = vector::empty<vector<u8>>();
  //   vector::push_back(&mut validators, b"0x6");
  //   vector::push_back(&mut validators, b"0x7");

  //   debug::print(&exists<Bridge>(admin_addr));
  //   initialize(&admin, validators);
  //   let bridge = borrow_global<Bridge>(admin_addr);

  //   assert!((vector::length(&bridge.validators) as u64) == 2, 1);
  //   assert!(*vector::borrow(&bridge.validators, 0) == b"@0x6", 2);
  //   assert!(*vector::borrow(&bridge.validators, 1) == b"@0x7", 3);
  //   assert!(bridge.deployed_collections == vector::empty<address>(), 4);

  //   initialize(&admin, validators);
  // }
}