module bridge::aptos_nft_bridge {
    use std::signer;
    use std::vector;
    use std::hash;
    use std::string::{Self, String};
    use std::bcs;
    use std::option::{Self};
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
    use aptos_token_objects::token::{Self, Token};
    use aptos_token_objects::royalty::{Self};
    use aptos_token_objects::collection::{Self, Collection};

    const E_ALREADY_INITIALIZED: u64 = 0;
    const E_NOT_BRIDGE_ADMIN: u64 = 1;
    const E_VALIDATORS_LENGTH_ZERO: u64 = 2;
    const E_VALIDATOR_ALREADY_EXIST: u64 = 3;
    const E_THERSHOLD_NOT_REACHED: u64 = 4;
    const E_INVALID_PK: u64 = 5;
    const E_INVALID_SIGNATURE: u64 = 6;
    const E_NOT_INITIALIZED: u64 = 7;
    const E_INVALID_FEE: u64 = 8;
    const E_NO_REWARDS_AVAILABLE: u64 = 9;
    const E_INVALID_NFT_TYPE: u64 = 10;
    const E_SIGNATURES_PUBLIC_KEYS_LENGTH_NOT_SAME: u64 = 11;
    const E_TOKEN_AMOUNT_IS_ZERO: u64 = 12;
    const E_CLAIM_ALREADY_PROCESSED: u64 = 13;
    const E_INVALID_DESTINATION_CHAIN: u64 = 14;
    const E_VALIDATOR_DOESNOT_EXIST: u64 = 15;
    const E_VALIDATOR_PENDING_REWARD_IS_ZERO: u64 = 16;
    const E_DESTINATION_CHAIN_SAME_AS_SOURCE: u64 = 17;
    const E_VALIDATOR_ALREADY_BLACKLISTED: u64 = 18;
    const E_NOT_A_VALIDATOR: u64 = 19;
    const E_VALIDATOR_BLACKLISTED: u64 = 20;
    const E_VALIDATOR_AND_PUBLIC_KEYS_NOT_EQUAL_LENGTH: u64 = 21;

    const TYPE_ERC721: vector<u8> = b"singular";
    const TYPE_ERC1155: vector<u8> = b"multiple";

    struct Validator has drop, store, copy {
        pending_reward: u64,
        addr: address
    }

    struct ProcessedClaims has key {
        claims: vector<vector<u8>>
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
        object: address
    }

    struct CollectionNftObject has drop, store, copy {
        collection_address: vector<u8>,
        token_id: u256
    }

    struct Bridge has key {
        validators: SimpleMap<vector<u8>, Validator>,
        signer_cap: SignerCapability,
        collection_objects: Table<CollectionObject, u256>, // TODO: we are not using it anywhere???
        nfts_counter: u64,
        original_to_duplicate_mapping: Table<OriginalToDuplicateKey, OriginalToDuplicateInfo>,
        duplicate_to_original_mapping: Table<DuplicateToOriginalKey, DuplicateToOriginalInfo>,
        nft_collection_tokens: SimpleMap<CollectionNftObject, address>,
        nft_collections_counter: SimpleMap<address, u256>, // collection_address -> token_id
        self_chain: vector<u8>,
        blacklisted_validators: SimpleMap<vector<u8>, bool>
    }

    struct SignatureInfo has drop {
        public_key: vector<u8>,
        signature: vector<u8>
    }

    struct ValidatorWithAddress has drop, copy {
        public_key: vector<u8>,
        addr: address
    }

    struct ClaimData has drop, copy {
        token_id: u256,
        source_chain: vector<u8>,
        destination_chain: vector<u8>,
        user: address,
        source_nft_contract_address: vector<u8>,
        name: String,
        royalty_percentage: u64,
        royalty_payee_address: address,
        metadata: String,
        transaction_hash: vector<u8>,
        token_amount: u256,
        nft_type: vector<u8>,
        fee: u64,
        symbol: String
    }

    #[event]
    struct AddNewValidatorEvent has drop, store {
        validator: vector<u8>
    }

    #[event]
    struct RewardValidatorEvent has drop, store {
        validator: vector<u8>
    }

    #[event]
    struct LockedEvent has drop, store {
        token_id: u256,
        destination_user_address: String,
        token_amount: u64,
        nft_type: vector<u8>,
        destination_chain: vector<u8>,
        source_chain: vector<u8>,
        collection_address: vector<u8>,
        token_address: address,
        metadata_uri: String
    }

    #[event]
    struct UnLock721Event has drop, store {
        to: address,
        token_id: u256
    }

    #[event]
    struct BlacklistValidatorEvent has drop, store {
        validator: vector<u8>
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

    fun check_signature(
        signature: vector<u8>, public_key: vector<u8>, data: vector<u8>
    ): bool {
        let pk = ed25519::new_unvalidated_public_key_from_bytes(public_key);
        let sig = ed25519::new_signature_from_bytes(signature);
        let verified = ed25519::signature_verify_strict(&sig, &pk, hash::sha2_256(data));
        verified
    }

    fun verify_signatures(
        signatures: vector<vector<u8>>,
        public_keys: &vector<vector<u8>>,
        data: vector<u8>,
        validators: &SimpleMap<vector<u8>, Validator>
    ): vector<vector<u8>> {
        let signatures_count = vector::length(&signatures);
        let percentage = 0;
        let uv = simple_map::new();

        for (i in 0..signatures_count) {
            let signature = *vector::borrow(&signatures, i);
            let public_key = *vector::borrow(public_keys, i);
            if (!simple_map::contains_key(validators, &public_key)) {
                // Not a validator
                continue
            };
            if (simple_map::contains_key(&uv, &public_key)) {
                // Signature for the validator is already counted
                continue
            };
            let sig_is_valid = check_signature(signature, public_key, data);
            if (sig_is_valid) {
                std::debug::print(&percentage);
                simple_map::add(&mut uv, public_key, true);
                percentage = percentage + 1;
            }
        };
        let validators_count = simple_map::length(validators);
        assert!(percentage >= ((validators_count * 2) / 3) + 1, E_THERSHOLD_NOT_REACHED);
        simple_map::keys(&uv)
    }

    fun retrieve_bridge_resource_address(): address acquires Bridge {
        let bridge_data = borrow_global<Bridge>(@bridge);
        let bridge_signer_from_cap =
            account::create_signer_with_capability(&bridge_data.signer_cap);
        signer::address_of(&bridge_signer_from_cap)
    }

    #[view]
    public fun owns_nft(owner: address, collection: String, name: String): bool {
        let token_addr = token::create_token_address(&owner, &collection, &name);
        object::object_exists<Token>(token_addr)
    }

    #[view]
    public fun token_id_from_nonce_and_collection(tid: u256, cid: vector<u8>): address acquires Bridge {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
        let bridge_data = borrow_global<Bridge>(@bridge);
        let taddr = simple_map::borrow(&bridge_data.nft_collection_tokens, &CollectionNftObject{
            collection_address: cid,
            token_id: tid
        });
        return *taddr
    }

    #[view]
    public fun validator_count(): u64 acquires Bridge {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
        let bridge_data = borrow_global<Bridge>(@bridge);
        return simple_map::length(&bridge_data.validators)
    }

    #[view]
    public fun has_validator(v: vector<u8>): bool acquires Bridge {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
        let bridge_data = borrow_global<Bridge>(@bridge);
        return simple_map::contains_key(&bridge_data.validators, &v)
    }

    public entry fun initialize(
        admin: &signer,
        val_public_key: vector<vector<u8>>,
        val_address: vector<address>,
        seed: vector<u8>,
        self_chain: vector<u8>
    ) {
        let admin_addr = signer::address_of(admin);
        assert!(admin_addr == @bridge, E_NOT_BRIDGE_ADMIN);
        assert!(!exists<Bridge>(admin_addr), E_ALREADY_INITIALIZED);
        let total_validators = vector::length(&val_public_key);
        assert!(total_validators > 0, E_VALIDATORS_LENGTH_ZERO);
        assert!(vector::length(&val_public_key) == vector::length(&val_address), E_VALIDATOR_AND_PUBLIC_KEYS_NOT_EQUAL_LENGTH);

        let (bridge_signer, signer_cap) = account::create_resource_account(admin, seed);
        let validators_to_add = simple_map::new<vector<u8>, Validator>();

        for (i in 0..total_validators) {
            let validator = *vector::borrow(&val_public_key, i);
            let validated_pk = ed25519::new_validated_public_key_from_bytes(validator);
            assert!(option::is_some(&validated_pk), E_INVALID_PK);
            simple_map::add(
                &mut validators_to_add, validator, Validator { pending_reward: 0, addr: *vector::borrow(&val_address, i)  }
            );
        };

        move_to<Bridge>(
            admin,
            Bridge {
                self_chain,
                validators: validators_to_add,
                signer_cap,
                collection_objects: table::new(),
                nfts_counter: 0,
                original_to_duplicate_mapping: table::new(),
                duplicate_to_original_mapping: table::new(),
                nft_collection_tokens: simple_map::create<CollectionNftObject, address>(),
                nft_collections_counter: simple_map::create<address, u256>(),
                blacklisted_validators: simple_map::new()
            }
        );

        move_to<ProcessedClaims>(
            &bridge_signer,
            ProcessedClaims {
                claims: vector::empty<vector<u8>>()
            }
        );
    }

    public entry fun add_validator(
        public_key: vector<u8>,
        addr: address,
        signatures: vector<vector<u8>>,
        public_keys: vector<vector<u8>>
    ) acquires Bridge {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
        let bridge_data = borrow_global_mut<Bridge>(@bridge);
        assert!(
            !simple_map::contains_key(&bridge_data.blacklisted_validators, &public_key),
            E_VALIDATOR_BLACKLISTED
        );
        let validated_pk = ed25519::new_validated_public_key_from_bytes(public_key);
        assert!(!option::is_none(&validated_pk), E_INVALID_PK);

        let signatures_count = vector::length(&signatures);
        let public_keys_count = vector::length(&public_keys);

        assert!(signatures_count > 0, E_VALIDATORS_LENGTH_ZERO);
        assert!(
            !simple_map::contains_key(&mut bridge_data.validators, &public_key),
            E_VALIDATOR_ALREADY_EXIST
        );
        assert!(
            signatures_count == public_keys_count,
            E_SIGNATURES_PUBLIC_KEYS_LENGTH_NOT_SAME
        );

        verify_signatures(
            signatures,
            &public_keys,
            bcs::to_bytes(&ValidatorWithAddress{
                public_key,
                addr
            }),
            &bridge_data.validators
        );

        event::emit(AddNewValidatorEvent { validator: public_key });
        simple_map::add(
            &mut bridge_data.validators, public_key, Validator { pending_reward: 0, addr: addr }
        );
    }

    public entry fun blacklist_validator(
        validator: vector<u8>,
        signatures: vector<vector<u8>>,
        public_keys: vector<vector<u8>>
    ) acquires Bridge {
        assert!(vector::length(&signatures) > 0, E_VALIDATORS_LENGTH_ZERO);
        assert!(
            vector::length(&signatures) == vector::length(&public_keys),
            E_SIGNATURES_PUBLIC_KEYS_LENGTH_NOT_SAME
        );
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
        let bridge_data = borrow_global_mut<Bridge>(@bridge);
        assert!(
            !simple_map::contains_key<vector<u8>, bool>(
                &bridge_data.blacklisted_validators, &validator
            ),
            E_VALIDATOR_ALREADY_BLACKLISTED
        );
        assert!(
            simple_map::contains_key(&bridge_data.validators, &validator),
            E_NOT_A_VALIDATOR
        );
        let validated_pk = ed25519::new_validated_public_key_from_bytes(validator);
        assert!(option::is_some((&validated_pk)), E_INVALID_PK);
        let payload = b"BLACKLIST:";
        vector::append(&mut payload, validator);
        verify_signatures(
            signatures,
            &public_keys,
            payload,
            &bridge_data.validators
        );
        simple_map::remove(&mut bridge_data.validators, &validator);
        simple_map::add(&mut bridge_data.blacklisted_validators, validator, true);
        event::emit(BlacklistValidatorEvent { validator });
    }

    public entry fun claim_validator_rewards(
        admin: signer, validator: vector<u8>
    ) acquires Bridge {

        let admin_addr = signer::address_of(&admin);
        assert!(admin_addr == @bridge, E_NOT_BRIDGE_ADMIN);
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

        let bridge_data = borrow_global_mut<Bridge>(@bridge);

        assert!(
            simple_map::contains_key(&mut bridge_data.validators, &validator),
            E_VALIDATOR_DOESNOT_EXIST
        );
        let validator_reward =
            simple_map::borrow_mut(&mut bridge_data.validators, &validator);

        assert!(validator_reward.pending_reward > 0, E_VALIDATOR_PENDING_REWARD_IS_ZERO);

        coin::transfer<AptosCoin>(&admin, validator_reward.addr, validator_reward.pending_reward);

        *validator_reward = Validator { pending_reward: 0, addr: validator_reward.addr};

        event::emit(RewardValidatorEvent { validator });
    }

    public entry fun lock_721(
        owner: &signer,
        token_address: address,
        destination_chain: vector<u8>,
        destination_user_address: String,
        collection_address: address,
        metadata_uri: String
    ) acquires Bridge {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
        let bridge_data = borrow_global_mut<Bridge>(@bridge);
        assert!(
            hash::sha2_256(bridge_data.self_chain) != hash::sha2_256(destination_chain),
            E_DESTINATION_CHAIN_SAME_AS_SOURCE
        );
        let bridge_signer_from_cap =
            account::create_signer_with_capability(&bridge_data.signer_cap);

        let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);

        let token_object = object::address_to_object<Token>(token_address);

        let nft_token_id: &u256;

        let collection_counter_intilaized =
            simple_map::contains_key(
                &bridge_data.nft_collections_counter, &collection_address
            );

        if (collection_counter_intilaized) {

            let nft_collection_counter =
                simple_map::borrow_mut(
                    &mut bridge_data.nft_collections_counter, &collection_address
                );
            nft_token_id = &(*nft_collection_counter + 1);
            *nft_collection_counter = *nft_collection_counter + 1;

        } else {
            simple_map::add(
                &mut bridge_data.nft_collections_counter, collection_address, 1
            );
            nft_token_id = &1;

        };

        simple_map::add(
            &mut bridge_data.nft_collection_tokens,
            CollectionNftObject {
                collection_address: bcs::to_bytes(&collection_address),
                token_id: *nft_token_id
            },
            token_address
        );

        let key_duplicate = DuplicateToOriginalKey {
            self_chain: bridge_data.self_chain,
            collection_address: bcs::to_bytes(&collection_address)
        };

        let is_original =
            !table::contains(
                &mut bridge_data.duplicate_to_original_mapping, key_duplicate
            );

        if (is_original) {
            event::emit(
                LockedEvent {
                    token_id: *nft_token_id,
                    destination_user_address,
                    token_amount: 1,
                    nft_type: TYPE_ERC721,
                    destination_chain,
                    source_chain: bridge_data.self_chain,
                    collection_address: bcs::to_bytes(&collection_address),
                    token_address,
                    metadata_uri
                }
            );

        } else {

            let original_collection_address =
                table::borrow(
                    &mut bridge_data.duplicate_to_original_mapping, key_duplicate
                );

            event::emit(
                LockedEvent {
                    token_id: *nft_token_id,
                    destination_user_address,
                    token_amount: 1,
                    nft_type: TYPE_ERC721,
                    destination_chain,
                    source_chain: original_collection_address.source_chain,
                    collection_address: original_collection_address.source_contract,
                    token_address,
                    metadata_uri
                }
            );
        };

        object::transfer(owner, token_object, bridge_resource_addr);
    }

    public entry fun lock_1155(
        owner: &signer,
        token_address: address,
        destination_chain: vector<u8>,
        destination_user_address: String,
        collection_address: address,
        amount: u64,
        metadata_uri: String
    ) acquires Bridge {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);
        assert!(amount > 0, E_TOKEN_AMOUNT_IS_ZERO);

        let bridge_data = borrow_global_mut<Bridge>(@bridge);

        assert!(
            hash::sha2_256(bridge_data.self_chain) != hash::sha2_256(destination_chain),
            E_DESTINATION_CHAIN_SAME_AS_SOURCE
        );

        let bridge_signer_from_cap =
            account::create_signer_with_capability(&bridge_data.signer_cap);

        let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);

        let token_object = object::address_to_object<Token>(token_address);
        let metadata = object::convert<Token, Metadata>(token_object);

        let nft_token_id: &u256;

        let collection_counter_intilaized =
            simple_map::contains_key(
                &bridge_data.nft_collections_counter, &collection_address
            );

        if (collection_counter_intilaized) {

            let nft_collection_counter =
                simple_map::borrow_mut(
                    &mut bridge_data.nft_collections_counter, &collection_address
                );
            nft_token_id = &(*nft_collection_counter + 1);
            *nft_collection_counter = *nft_collection_counter + 1;

        } else {
            simple_map::add(
                &mut bridge_data.nft_collections_counter, collection_address, 1
            );
            nft_token_id = &1;

        };

        simple_map::add(
            &mut bridge_data.nft_collection_tokens,
            CollectionNftObject {
                collection_address: bcs::to_bytes(&collection_address),
                token_id: *nft_token_id
            },
            token_address
        );

        let key_duplicate = DuplicateToOriginalKey {
            self_chain: bridge_data.self_chain,
            collection_address: bcs::to_bytes(&collection_address)
        };

        let is_original =
            !table::contains(
                &mut bridge_data.duplicate_to_original_mapping, key_duplicate
            );

        if (is_original) {
            event::emit(
                LockedEvent {
                    token_id: *nft_token_id,
                    destination_user_address,
                    token_amount: amount,
                    nft_type: TYPE_ERC1155,
                    destination_chain,
                    source_chain: bridge_data.self_chain,
                    collection_address: bcs::to_bytes(&collection_address),
                    token_address,
                    metadata_uri
                }
            );

        } else {

            let original_collection_address =
                table::borrow(
                    &mut bridge_data.duplicate_to_original_mapping, key_duplicate
                );

            event::emit(
                LockedEvent {
                    token_id: *nft_token_id,
                    destination_user_address,
                    token_amount: amount,
                    nft_type: TYPE_ERC1155,
                    destination_chain,
                    source_chain: original_collection_address.source_chain,
                    collection_address: original_collection_address.source_contract,
                    token_address,
                    metadata_uri
                }
            );

        };

        primary_fungible_store::transfer(
            owner, metadata, bridge_resource_addr, amount
        );
    }

    public entry fun claim_721(
        user: &signer,
        destination_user_address: address,
        name: String, // collection name
        uri: String, //token uri
        royalty_percentage: u64,
        royalty_payee_address: address,
        fee: u64,
        signatures: vector<vector<u8>>,
        public_keys: vector<vector<u8>>,
        destination_chain: vector<u8>,
        source_chain: vector<u8>,
        source_nft_contract_address: vector<u8>,
        token_id: u256,
        transaction_hash: vector<u8>,
        nft_type: vector<u8>,
        metadata: String, // we are not using it.
        symbol: String // not used here. used in 1155 inside primary store
    ) acquires Bridge, ProcessedClaims {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

        assert!(
            hash::sha2_256(nft_type) == hash::sha2_256(TYPE_ERC721), E_INVALID_NFT_TYPE
        );

        let bridge_data = borrow_global_mut<Bridge>(@bridge);
        assert!(
            hash::sha2_256(bridge_data.self_chain) == hash::sha2_256(destination_chain),
            E_INVALID_DESTINATION_CHAIN
        );

        let data = ClaimData {
            token_id,
            source_chain,
            destination_chain,
            user: destination_user_address,
            source_nft_contract_address,
            name,
            royalty_percentage,
            royalty_payee_address,
            metadata,
            transaction_hash,
            token_amount: 0,
            nft_type,
            fee,
            symbol
        };

        let bridge_signer_from_cap =
            account::create_signer_with_capability(&bridge_data.signer_cap);
        let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);
        let processed_claims = borrow_global_mut<ProcessedClaims>(bridge_resource_addr);

        let data_hash = hash::sha2_256(bcs::to_bytes(&data));
        let duplicate_claim = vector::contains(&processed_claims.claims, &data_hash);

        assert!(!duplicate_claim, E_CLAIM_ALREADY_PROCESSED);

        vector::push_back(&mut processed_claims.claims, data_hash);

        verify_signatures(
            signatures,
            &public_keys,
            bcs::to_bytes(&data),
            &bridge_data.validators
        );

        coin::transfer<AptosCoin>(user, @bridge, fee);

        reward_validators(fee, &public_keys, &mut bridge_data.validators);

        let is_locked: bool = false;

        let token_exists =
            simple_map::contains_key(
                &bridge_data.nft_collection_tokens,
                &CollectionNftObject {
                    collection_address: source_nft_contract_address,
                    token_id
                }
            );

        let token_address: &address = &@0x0;

        if (token_exists) {
            token_address = simple_map::borrow(
                &bridge_data.nft_collection_tokens,
                &CollectionNftObject {
                    collection_address: source_nft_contract_address,
                    token_id
                }
            );

            let resource_object_exists = object::object_exists<Token>(*token_address);

            if (resource_object_exists) {
                let token_object_resource =
                    object::address_to_object<Token>(*token_address);
                if (object::owner(token_object_resource) == bridge_resource_addr) {
                    is_locked = true;
                }
            };

        };

        if (*token_address != @0x0 && is_locked) {

            let token_object = object::address_to_object<Token>(*token_address);

            event::emit(UnLock721Event { to: destination_user_address, token_id });

            object::transfer(
                &bridge_signer_from_cap, token_object, destination_user_address
            );

            event::emit(
                Claim721Event {
                    token_id,
                    transaction_hash,
                    source_chain,
                    nft_contract: source_nft_contract_address
                }
            );

        } else {

            let collection_address =
                collection::create_collection_address(&bridge_resource_addr, &name);

            let collection_exists = object::object_exists<Collection>(collection_address);

            if (!collection_exists) {
                collection::create_unlimited_collection(
                    &bridge_signer_from_cap,
                    name,
                    name,
                    option::none(),
                    string::utf8(b"")
                );
            };

            let royalty = royalty::create(royalty_percentage, 100, royalty_payee_address);

            token::create_named_token(
                &bridge_signer_from_cap,
                name,
                name,
                name,
                option::some(royalty),
                uri
            );

            let created_token_addr =
                token::create_token_address(&bridge_resource_addr, &name, &name);

            let token_object = object::address_to_object<Token>(created_token_addr);

            object::transfer(
                &bridge_signer_from_cap, token_object, destination_user_address
            );

            let key = OriginalToDuplicateKey {
                source_chain,
                source_contract: source_nft_contract_address
            };

            let is_duplicate =
                !table::contains(&mut bridge_data.original_to_duplicate_mapping, key);

            if (is_duplicate) {

                let value = OriginalToDuplicateInfo {
                    self_chain: bridge_data.self_chain,
                    collection_address: bcs::to_bytes(&collection_address)
                };

                let key_duplicate = DuplicateToOriginalKey {
                    self_chain: bridge_data.self_chain,
                    collection_address: bcs::to_bytes(&collection_address)
                };

                let value_duplicate = DuplicateToOriginalInfo {
                    source_chain,
                    source_contract: bcs::to_bytes(&source_nft_contract_address)
                };

                table::add(&mut bridge_data.original_to_duplicate_mapping, key, value);

                table::add(
                    &mut bridge_data.duplicate_to_original_mapping,
                    key_duplicate,
                    value_duplicate
                );
            };

            event::emit(
                Claim721Event {
                    token_id,
                    transaction_hash,
                    source_chain,
                    nft_contract: bcs::to_bytes(&collection_address)
                }
            );

        };

    }

    public entry fun claim_1155(
        user: &signer,
        destination_user_address: address,
        name: String,
        uri: String,
        royalty_percentage: u64,
        royalty_payee_address: address,
        fee: u64,
        signatures: vector<vector<u8>>,
        public_keys: vector<vector<u8>>,
        destination_chain: vector<u8>,
        source_chain: vector<u8>,
        source_nft_contract_address: vector<u8>,
        token_id: u256,
        transaction_hash: vector<u8>,
        nft_type: vector<u8>,
        metadata: String,
        symbol: String,
        amount: u64
    ) acquires Bridge, ProcessedClaims {
        assert!(exists<Bridge>(@bridge), E_NOT_INITIALIZED);

        assert!(
            hash::sha2_256(nft_type) == hash::sha2_256(TYPE_ERC1155), E_INVALID_NFT_TYPE
        );

        let bridge_data = borrow_global_mut<Bridge>(@bridge);
        assert!(
            hash::sha2_256(bridge_data.self_chain) == hash::sha2_256(destination_chain),
            E_INVALID_DESTINATION_CHAIN
        );

        let data = ClaimData {
            token_id,
            source_chain,
            destination_chain,
            user: destination_user_address,
            source_nft_contract_address,
            name,
            royalty_percentage,
            royalty_payee_address,
            metadata,
            transaction_hash,
            token_amount: (amount as u256),
            nft_type,
            fee,
            symbol
        };

        let bridge_signer_from_cap =
            account::create_signer_with_capability(&bridge_data.signer_cap);
        let bridge_resource_addr = signer::address_of(&bridge_signer_from_cap);
        let processed_claims = borrow_global_mut<ProcessedClaims>(bridge_resource_addr);

        let data_hash = hash::sha2_256(bcs::to_bytes(&data));
        let duplicate_claim = vector::contains(&processed_claims.claims, &data_hash);
        assert!(!duplicate_claim, E_CLAIM_ALREADY_PROCESSED);

        vector::push_back(&mut processed_claims.claims, data_hash);

        let to_reward = verify_signatures(
            signatures,
            &public_keys,
            bcs::to_bytes(&data),
            &bridge_data.validators
        );

        coin::transfer<AptosCoin>(user, @bridge, fee);

        reward_validators(fee, &to_reward, &mut bridge_data.validators);

        let is_locked: bool = false;

        let token_exists =
            simple_map::contains_key(
                &bridge_data.nft_collection_tokens,
                &CollectionNftObject {
                    collection_address: source_nft_contract_address,
                    token_id
                }
            );

        let token_address: &address = &@0x0;

        if (token_exists) {
            token_address = simple_map::borrow(
                &bridge_data.nft_collection_tokens,
                &CollectionNftObject {
                    collection_address: source_nft_contract_address,
                    token_id
                }
            );

            let resource_object_exists = object::object_exists<Token>(*token_address);

            if (resource_object_exists) {
                let token_object_resource =
                    object::address_to_object<Token>(*token_address);
                if (object::owner(token_object_resource) == bridge_resource_addr) {
                    is_locked = true;
                }
            };

        };

        if (*token_address != @0x0 && is_locked) {

            let token_object = object::address_to_object<Token>(*token_address);
            let metadata = object::convert<Token, Metadata>(token_object);

            let store =
                primary_fungible_store::ensure_primary_store_exists(
                    bridge_resource_addr, metadata
                );
            let balance_of_tokens = fungible_asset::balance(store);

            if (balance_of_tokens >= amount) {
                primary_fungible_store::transfer(
                    &bridge_signer_from_cap,
                    token_object,
                    destination_user_address,
                    amount
                );

                event::emit(
                    UnLock1155Event { to: destination_user_address, token_id, amount }
                );

            } else {

                let to_mint = amount - balance_of_tokens;

                primary_fungible_store::transfer(
                    &bridge_signer_from_cap,
                    token_object,
                    destination_user_address,
                    balance_of_tokens
                );
                event::emit(
                    UnLock1155Event {
                        to: destination_user_address,
                        token_id,
                        amount: balance_of_tokens
                    }
                );

                let royalty =
                    royalty::create(royalty_percentage, 1000, royalty_payee_address);

                let new_nft_constructor_ref =
                    &token::create_named_token(
                        &bridge_signer_from_cap,
                        name,
                        name,
                        name,
                        option::some(royalty),
                        uri
                    );

                // Make this nft token fungible so there can multiple instances of it.
                primary_fungible_store::create_primary_store_enabled_fungible_asset(
                    new_nft_constructor_ref,
                    option::some((to_mint as u128)),
                    name,
                    symbol,
                    0, // NFT cannot be divided so decimals is 0,
                    string::utf8(b""),
                    string::utf8(b"")
                );

                let mint_ref =
                    &fungible_asset::generate_mint_ref(new_nft_constructor_ref);
                let fa = fungible_asset::mint(mint_ref, amount);
                primary_fungible_store::deposit(destination_user_address, fa);
            };

            event::emit(
                Claim1155Event {
                    token_id,
                    transaction_hash,
                    source_chain,
                    amount,
                    nft_contract: source_nft_contract_address
                }
            );

        } else {
            let collection_address =
                collection::create_collection_address(&bridge_resource_addr, &name);

            let collection_exists = object::object_exists<Collection>(collection_address);

            if (!collection_exists) {
                collection::create_unlimited_collection(
                    &bridge_signer_from_cap,
                    name,
                    name,
                    option::none(),
                    string::utf8(b"")
                );
            };

            let royalty = royalty::create(royalty_percentage, 100, royalty_payee_address);

            let new_nft_constructor_ref =
                &token::create_named_token(
                    &bridge_signer_from_cap,
                    name,
                    name,
                    name,
                    option::some(royalty),
                    uri
                );

            // Make this nft token fungible so there can multiple instances of it.
            primary_fungible_store::create_primary_store_enabled_fungible_asset(
                new_nft_constructor_ref,
                option::some((amount as u128)),
                name,
                symbol,
                0, // NFT cannot be divided so decimals is 0,
                string::utf8(b""),
                string::utf8(b"")
            );

            let mint_ref = &fungible_asset::generate_mint_ref(new_nft_constructor_ref);
            let fa = fungible_asset::mint(mint_ref, amount);
            primary_fungible_store::deposit(destination_user_address, fa);

            let key = OriginalToDuplicateKey {
                source_chain,
                source_contract: source_nft_contract_address
            };

            let is_duplicate =
                table::contains(&mut bridge_data.original_to_duplicate_mapping, key);

            if (!is_duplicate) {

                let value = OriginalToDuplicateInfo {
                    self_chain: bridge_data.self_chain,
                    collection_address: bcs::to_bytes(&collection_address)
                };

                let key_duplicate = DuplicateToOriginalKey {
                    self_chain: bridge_data.self_chain,
                    collection_address: bcs::to_bytes(&collection_address)
                };

                let value_duplicate = DuplicateToOriginalInfo {
                    source_chain,
                    source_contract: bcs::to_bytes(&source_nft_contract_address)
                };

                table::add(&mut bridge_data.original_to_duplicate_mapping, key, value);

                table::add(
                    &mut bridge_data.duplicate_to_original_mapping,
                    key_duplicate,
                    value_duplicate
                );
            };

            event::emit(
                Claim721Event {
                    token_id,
                    transaction_hash,
                    source_chain,
                    nft_contract: bcs::to_bytes(&collection_address)
                }
            );
        };

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

        for (i in 0..total_validators_to_reward) {
            let validator = vector::borrow(validators_to_reward, i);
            let validator_reward = simple_map::borrow_mut(all_validators, validator);
            *validator_reward = Validator {
                pending_reward: validator_reward.pending_reward + fee_per_validator, addr: validator_reward.addr
            };
        };
    }
}
