#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// This code imports necessary aspects of external crates that we will use in our contract code.
extern crate alloc;
mod endpoints;
mod errors;
mod events;
mod keys;
mod structs;
mod utils;

use core::convert::TryInto;

// Importing Rust types.
use alloc::boxed::Box;
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
// Importing aspects of the Casper platform.
use casper_contract::contract_api::system::{
    create_purse, get_purse_balance, transfer_from_purse_to_account,
    transfer_from_purse_to_public_key, transfer_from_purse_to_purse,
};
use casper_contract::{
    contract_api::{
        runtime::{self},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
// Importing specific Casper types.
use crate::events::{
    BlackListValidator, Claimed, DeployCollection, DeployStorage, Locked, RewardValidator, UnLock,
};
use crate::structs::{
    ClaimData, DuplicateToOriginalContractInfo, OriginalToDuplicateContractInfo, Receiver,
};
use casper_types::account::AccountHash;
use casper_types::{
    bytesrepr::ToBytes,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    CLType, ContractHash, Key, Parameter, PublicKey as CPublicKey, URef, U256, U512,
};
use common::collection::{mint, owner_of, register, transfer, TokenIdentifier};
use common::storage::unlock_token;
use ed25519_dalek::{PublicKey as EPublicKey, Signature, Verifier};
use endpoints::*;
use errors::BridgeError;
use events::AddNewValidator;
use keys::*;
use sha2::{Digest, Sha256, Sha512};
use structs::Validator;

type Sigs = (CPublicKey, [u8; 64]);

pub fn get_service_acc_hash() -> AccountHash {
    let service_ref = utils::get_uref(
        KEY_SERVICE_ADDRESS,
        BridgeError::MissingServiceAddressRef,
        BridgeError::InvalidServiceAddressRef,
    );
    let receiver_acc_hash: CPublicKey = storage::read_or_revert(service_ref);
    receiver_acc_hash.to_account_hash()
}
pub fn transfer_tx_fees(amount: U512, sender_purse: URef, to: Receiver) {
    match to {
        Receiver::Sender => {
            let caller = runtime::get_caller();
            transfer_from_purse_to_account(sender_purse, caller, amount, None).unwrap_or_revert();
        }
        Receiver::Service => {
            transfer_from_purse_to_account(sender_purse, get_service_acc_hash(), amount, None)
                .unwrap_or_revert();
        }
        Receiver::Bridge => {
            let this_contract_purse_ref = utils::get_uref(
                KEY_PURSE,
                BridgeError::MissingThisPurseRef,
                BridgeError::InvalidThisPurseRef,
            );
            let purse: URef = this_contract_purse_ref;

            transfer_from_purse_to_purse(sender_purse, purse, amount, None).unwrap_or_revert();
        }
    }
}
fn matches_current_chain(destination_chain: String, self_chain: String) {
    if destination_chain != self_chain {
        runtime::revert(BridgeError::InvalidDestinationChain);
    }
}
/// Ed25519 Signature verification logic.
fn verify_signatures(data: Vec<u8>, signatures: Vec<Sigs>) -> Vec<CPublicKey> {
    let mut percentage: u64 = 0;

    let mut validators_to_reward: Vec<CPublicKey> = vec![];
    let mut uv: Vec<CPublicKey> = [].to_vec();

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );

    let mut hasher = Sha512::new();
    hasher.update(data);
    let hash = hasher.finalize();

    for arg in signatures.into_iter() {
        let key = arg
            .0
            .to_bytes()
            .unwrap()
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<u8>>();
        let sig = arg.1.to_bytes().unwrap();

        let signature = Signature::new(
            sig.as_slice()
                .try_into()
                .map_err(|_| BridgeError::FailedToPrepareSignature)
                .unwrap_or_revert(),
        );

        let key = EPublicKey::from_bytes(key.as_slice())
            .map_err(|_| BridgeError::FailedToPreparePublicKey)
            .unwrap_or_revert();

        let valid = key.verify(&hash, &signature);

        if valid.is_ok() {
            let validator_exists =
                storage::dictionary_get::<Validator>(validators_dict_ref, &arg.0.to_string())
                    .unwrap_or(None);

            match validator_exists {
                Some(v) => {
                    if v.added && !uv.contains(&arg.0) {
                        percentage = percentage + 1;
                        uv.push(arg.0.clone());
                        validators_to_reward.push(arg.0);
                    }
                }
                None => {}
            }
        }
    }
    let validators_count_ref = utils::get_uref(
        KEY_VALIDATORS_COUNT,
        BridgeError::MissingValidatorsCountRef,
        BridgeError::InvalidValidatorsCountRef,
    );
    let validators_count: u64 = storage::read_or_revert(validators_count_ref);

    if percentage >= (((validators_count * 2) / 3) + 1) {
        validators_to_reward
    } else {
        runtime::revert(BridgeError::ThresholdNotReached);
    }
}
fn check_storage(
    storage_dict_ref: URef,
    key: &str,
    newly_created_storage_option: Option<ContractHash>,
    nft_sender_address_option: Option<AccountHash>,
    token_id: TokenIdentifier,
    destination_chain: String,
    destination_user_address: String,
    source_nft_contract_address: ContractHash,
    metadata_uri: String,
) -> bool {
    let storage_address_option =
        storage::dictionary_get::<ContractHash>(storage_dict_ref, key).unwrap_or(None);
    match storage_address_option {
        Some(v) => {
            // TRANSFER TO STORAGE
            transfer_to_storage(
                v,
                nft_sender_address_option,
                source_nft_contract_address,
                token_id,
            );
            true
        }
        None => {
            if newly_created_storage_option.is_some() {
                // SAVE STORAGE DATA
                let storage = newly_created_storage_option
                    .unwrap_or_revert_with(BridgeError::FailedToUnWrapStorageAddress);

                storage::dictionary_put(storage_dict_ref, key, storage);

                transfer_to_storage(
                    storage,
                    nft_sender_address_option,
                    source_nft_contract_address,
                    token_id,
                );
                true
            } else {
                // EMIT CREATE STORAGE EVENT
                casper_event_standard::emit(DeployStorage::new(
                    token_id,
                    destination_chain,
                    destination_user_address,
                    source_nft_contract_address.to_string(),
                    metadata_uri,
                    runtime::get_caller().to_string(),
                ));
                false
                // runtime::ret(CLValue::from_t("deploy_storage").unwrap_or_revert());
            }
        }
    }
}
fn transfer_to_storage(
    storage_address: ContractHash,
    nft_sender_address_option: Option<AccountHash>,
    source_nft_contract_address: ContractHash,
    token_id: TokenIdentifier,
) {
    let from_address;

    if nft_sender_address_option.is_some() {
        from_address = Key::from(nft_sender_address_option.unwrap());
    } else {
        from_address = Key::from(runtime::get_caller());
    }

    register(source_nft_contract_address, from_address);

    register(source_nft_contract_address, Key::from(storage_address));

    transfer(
        source_nft_contract_address,
        from_address,
        Key::from(storage_address),
        token_id,
    );
}
fn reward_validators(fee: U512, validators: Vec<CPublicKey>) {
    if fee <= U512::from(0) {
        runtime::revert(BridgeError::InvalidFee);
    }

    let this_contract_purse_ref = utils::get_uref(
        KEY_PURSE,
        BridgeError::MissingThisPurseRef,
        BridgeError::InvalidThisPurseRef,
    );

    let purse: URef = this_contract_purse_ref;

    let total_rewards = get_purse_balance(purse).unwrap_or_else(|| U512::from(0));

    if total_rewards < fee {
        runtime::revert(BridgeError::NoRewardsAvailable);
    }

    let fee_per_validator = total_rewards / validators.len();

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );

    for arg in validators {
        let validator_option =
            storage::dictionary_get::<Validator>(validators_dict_ref, &arg.to_string())
                .unwrap_or_revert_with(BridgeError::FailedToGetValidatorForReward);
        match validator_option {
            Some(mut v) => {
                v.pending_rewards += fee_per_validator;
            }
            None => {
                runtime::revert(BridgeError::FailedToGetValidatorForReward);
            }
        }
    }
}
#[no_mangle]
pub extern "C" fn init() {
    if utils::named_uref_exists(INITIALIZED) {
        runtime::revert(BridgeError::AlreadyInitialized);
    }

    let validator: CPublicKey = runtime::get_named_arg(ARG_VALIDATOR);
    let chain_type: String = runtime::get_named_arg(ARG_CHAIN_TYPE);
    let collection_deployer: ContractHash = runtime::get_named_arg(ARG_COLLECTION_DEPLOYER);
    let storage_deployer: ContractHash = runtime::get_named_arg(ARG_STORAGE_DEPLOYER);
    let service_account: CPublicKey = runtime::get_named_arg(ARG_SERVICE_ADDRESS);
    let self_hash: ContractHash = runtime::get_named_arg(ARG_SELF_HASH);

    let storage_deploy_fee: U512 = runtime::get_named_arg(ARG_STORAGE_DEPLOY_FEE);
    let collection_deploy_fee: U512 = runtime::get_named_arg(ARG_COLLECTION_DEPLOY_FEE);
    let claim_fee: U512 = runtime::get_named_arg(ARG_CLAIM_FEE);

    runtime::put_key(INITIALIZED, storage::new_uref(true).into());
    runtime::put_key(KEY_CHAIN_TYPE, storage::new_uref(chain_type).into());
    runtime::put_key(
        KEY_COLLECTION_DEPLOYER,
        storage::new_uref(collection_deployer).into(),
    );
    runtime::put_key(
        KEY_STORAGE_DEPLOYER,
        storage::new_uref(storage_deployer).into(),
    );
    runtime::put_key(
        KEY_SERVICE_ADDRESS,
        storage::new_uref(service_account).into(),
    );
    runtime::put_key(KEY_TYPE_ERC721, storage::new_uref("singular").into());
    runtime::put_key(KEY_SELF_HASH, storage::new_uref(self_hash).into());
    runtime::put_key(KEY_PURSE, create_purse().into());
    runtime::put_key(
        KEY_STORAGE_DEPLOY_FEE,
        storage::new_uref(storage_deploy_fee).into(),
    );
    runtime::put_key(
        KEY_COLLECTION_DEPLOY_FEE,
        storage::new_uref(collection_deploy_fee).into(),
    );
    runtime::put_key(KEY_CLAIM_FEE, storage::new_uref(claim_fee).into());

    // INITIALIZING BOOTSTRAP VALIDATOR
    let validators_dict = storage::new_dictionary(KEY_VALIDATORS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    let validator_public_key = &validator.to_string();

    storage::dictionary_put(
        validators_dict,
        &validator_public_key,
        Validator {
            added: true,
            pending_rewards: U512::from(0),
        },
    );

    // INITIALIZING VALIDATOR COUNT
    runtime::put_key(KEY_VALIDATORS_COUNT, storage::new_uref(1u64).into());

    // INITIALIZING BLACKLIST VALIDATORS DICT
    storage::new_dictionary(KEY_BLACKLIST_VALIDATORS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING UNIQUE IDENTIFIERS DICT
    storage::new_dictionary(KEY_UNIQUE_IDENTIFIERS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING ORIGINAL TO DUPLICATE DICT
    storage::new_dictionary(KEY_ORIGINAL_TO_DUPLICATE_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING DUPLICATE TO ORIGINAL DICT
    storage::new_dictionary(KEY_DUPLICATE_TO_ORIGINAL_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING ORIGINAL STORAGE DICT
    storage::new_dictionary(KEY_ORIGINAL_STORAGE_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING DUPLICATE STORAGE DICT
    storage::new_dictionary(KEY_DUPLICATE_STORAGE_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    utils::init_events();
}
#[no_mangle]
pub extern "C" fn add_validator() {
    let new_validator_public_key: CPublicKey = utils::get_named_arg_with_user_errors(
        ARG_NEW_VALIDATOR_PUBLIC_KEY,
        BridgeError::MissingArgumentNewValidator,
        BridgeError::InvalidArgumentNewValidator,
    )
    .unwrap_or_revert();

    let signatures: Vec<Sigs> = utils::get_named_arg_with_user_errors(
        ARG_SIGNATURES,
        BridgeError::MissingArgumentSignatures,
        BridgeError::InvalidArgumentSignatures,
    )
    .unwrap_or_revert();

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );

    let data = new_validator_public_key
        .to_bytes()
        .unwrap()
        .iter()
        .skip(1)
        .cloned()
        .collect::<Vec<u8>>();

    verify_signatures(data, signatures);

    let validators_count_ref = utils::get_uref(
        KEY_VALIDATORS_COUNT,
        BridgeError::MissingValidatorsCountRef,
        BridgeError::InvalidValidatorsCountRef,
    );
    let validators_count: u64 = storage::read_or_revert(validators_count_ref);

    storage::dictionary_put(
        validators_dict_ref,
        &new_validator_public_key.to_string(),
        Validator {
            added: true,
            pending_rewards: U512::from(0),
        },
    );

    storage::write(validators_count_ref, validators_count + 1);

    casper_event_standard::emit(AddNewValidator::new(new_validator_public_key));
}
#[no_mangle]
pub extern "C" fn blacklist_validator() {
    let validator_public_key: CPublicKey = utils::get_named_arg_with_user_errors(
        ARG_VALIDATOR_PUBLIC_KEY,
        BridgeError::MissingArgumentValidator,
        BridgeError::InvalidArgumentValidator,
    )
    .unwrap_or_revert();

    let signatures: Vec<Sigs> = utils::get_named_arg_with_user_errors(
        ARG_SIGNATURES,
        BridgeError::MissingArgumentSignatures,
        BridgeError::InvalidArgumentSignatures,
    )
    .unwrap_or_revert();

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );

    let blacklist_validators_dict_ref = utils::get_uref(
        KEY_BLACKLIST_VALIDATORS_DICT,
        BridgeError::MissingBlackListValidatorsDictRef,
        BridgeError::InvalidBlackListValidatorsDictRef,
    );

    storage::dictionary_get::<Validator>(validators_dict_ref, &validator_public_key.to_string())
        .unwrap_or_revert_with(BridgeError::ValidatorNotAdded);
    let blacklist_exists = storage::dictionary_get::<bool>(
        blacklist_validators_dict_ref,
        &validator_public_key.to_string(),
    )
    .unwrap_or(Option::from(false));

    match blacklist_exists {
        Some(_) => {
            runtime::revert(BridgeError::AlreadyBlacklisted);
        }
        None => {}
    }

    let data = validator_public_key
        .to_bytes()
        .unwrap()
        .iter()
        .skip(1)
        .cloned()
        .collect::<Vec<u8>>();

    verify_signatures(data, signatures);

    let validators_count_ref = utils::get_uref(
        KEY_VALIDATORS_COUNT,
        BridgeError::MissingValidatorsCountRef,
        BridgeError::InvalidValidatorsCountRef,
    );
    let validators_count: u64 = storage::read_or_revert(validators_count_ref);

    storage::dictionary_put(
        validators_dict_ref,
        &validator_public_key.to_string(),
        Option::<String>::None,
    );

    storage::dictionary_put(
        blacklist_validators_dict_ref,
        &validator_public_key.to_string(),
        true,
    );
    storage::write(validators_count_ref, validators_count - 1);

    casper_event_standard::emit(BlackListValidator::new(validator_public_key));
}
#[no_mangle]
pub extern "C" fn claim_reward_validator() {
    let validator_public_key: CPublicKey = utils::get_named_arg_with_user_errors(
        ARG_VALIDATOR_PUBLIC_KEY,
        BridgeError::MissingArgumentValidator,
        BridgeError::InvalidArgumentValidator,
    )
    .unwrap_or_revert();

    let this_contract_purse_ref = utils::get_uref(
        KEY_PURSE,
        BridgeError::MissingThisPurseRef,
        BridgeError::InvalidThisPurseRef,
    );

    let purse: URef = this_contract_purse_ref;

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );
    let validator_option = storage::dictionary_get::<Validator>(
        validators_dict_ref,
        &validator_public_key.to_string(),
    )
    .unwrap_or(None);
    let rewards;
    match validator_option {
        Some(v) => {
            if !v.added {
                runtime::revert(BridgeError::ValidatorDoesNotExist);
            } else {
                rewards = v.pending_rewards;
                let res = transfer_from_purse_to_public_key(
                    purse,
                    validator_public_key.clone(),
                    rewards,
                    None,
                );
                match res {
                    Ok(_) => {
                        storage::dictionary_put(
                            validators_dict_ref,
                            &validator_public_key.to_string(),
                            Validator {
                                added: true,
                                pending_rewards: U512::from(0),
                            },
                        );
                        casper_event_standard::emit(RewardValidator::new(validator_public_key));
                    }
                    Err(e) => {
                        runtime::revert(e);
                    }
                }
            }
        }
        None => {
            runtime::revert(BridgeError::ValidatorNotAdded);
        }
    }
}
#[no_mangle]
pub extern "C" fn lock() {
    // RUNTIME ARGS START
    let token_id: TokenIdentifier = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        BridgeError::MissingArgumentTokenId,
        BridgeError::InvalidArgumentTokenId,
    )
    .unwrap_or_revert();

    let destination_chain: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_CHAIN,
        BridgeError::MissingArgumentDestinationChain,
        BridgeError::InvalidArgumentDestinationChain,
    )
    .unwrap_or_revert();

    let destination_user_address: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_USER_ADDRESS,
        BridgeError::MissingArgumentDestinationUserAddress,
        BridgeError::InvalidArgumentDestinationUserAddress,
    )
    .unwrap_or_revert();

    let source_nft_contract_address: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_NFT_CONTRACT_ADDRESS,
        BridgeError::MissingArgumentSourceNftContractAddress,
        BridgeError::InvalidArgumentSourceNftContractAddress,
    )
    .unwrap_or_revert();

    let metadata_uri: String = utils::get_named_arg_with_user_errors(
        ARG_METADATA,
        BridgeError::MissingArgumentMetadata,
        BridgeError::InvalidArgumentMetadata,
    )
    .unwrap_or_revert();

    let sender_purse: URef = utils::get_named_arg_with_user_errors(
        ARG_SENDER_PURSE,
        BridgeError::MissingArgumentSenderPurse,
        BridgeError::InvalidArgumentSenderPurse,
    )
    .unwrap_or_revert();

    let amount: U512 = utils::get_named_arg_with_user_errors(
        ARG_AMOUNT,
        BridgeError::MissingArgumentAmount,
        BridgeError::InvalidArgumentAmount,
    )
    .unwrap_or_revert();

    // RUNTIME ARGS END

    let storage_deploy_fee_ref = utils::get_uref(
        KEY_STORAGE_DEPLOY_FEE,
        BridgeError::MissingStorageDeployFeeRef,
        BridgeError::InvalidStorageDeployFeeRef,
    );

    let storage_deploy_fee: U512 = storage::read_or_revert(storage_deploy_fee_ref);

    // CHECK STORAGE DEPLOY FEE
    if amount < storage_deploy_fee {
        runtime::revert(BridgeError::StorageFeeLessThanSentAmount);
    }

    let self_chain_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );

    let self_chain: String = storage::read_or_revert(self_chain_ref);

    let nft_type_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );

    let nft_type: String = storage::read_or_revert(nft_type_ref);

    let duplicate_to_original_dict_ref = utils::get_uref(
        KEY_DUPLICATE_TO_ORIGINAL_DICT,
        BridgeError::MissingDTODictRef,
        BridgeError::InvalidOTDDictRef,
    );

    let mut data_key = source_nft_contract_address.to_bytes().unwrap();

    let self_chain_b = self_chain.to_bytes().unwrap();

    data_key.extend(self_chain_b);

    let mut hasher = Sha256::new();

    hasher.update(data_key);

    let hash = hasher.finalize();

    let binding = hex::encode(hash);

    let key = binding.as_str();

    let original_collection_address_option = storage::dictionary_get::<
        DuplicateToOriginalContractInfo,
    >(duplicate_to_original_dict_ref, key)
    .unwrap_or(None);

    match original_collection_address_option {
        Some(v) => {
            // notOriginal
            let duplicate_storage_dict_ref = utils::get_uref(
                KEY_DUPLICATE_STORAGE_DICT,
                BridgeError::MissingDSRef,
                BridgeError::InvalidDSRef,
            );
            let is_event = check_storage(
                duplicate_storage_dict_ref,
                key,
                Option::None,
                Option::None,
                token_id.clone(),
                destination_chain.clone(),
                destination_user_address.clone(),
                source_nft_contract_address,
                metadata_uri.clone(),
            );
            if !is_event {
                // SEND MONEY TO SERVICE
                transfer_tx_fees(amount, sender_purse, Receiver::Service);
            }
            if is_event {
                transfer_tx_fees(amount, sender_purse, Receiver::Sender);
                casper_event_standard::emit(Locked::new(
                    token_id,
                    destination_chain,
                    destination_user_address,
                    v.contract_address.to_string(),
                    U256::from(1),
                    nft_type,
                    v.chain,
                    metadata_uri,
                ));
            }
        }
        None => {
            // isOriginal
            let original_storage_dict_ref = utils::get_uref(
                KEY_ORIGINAL_STORAGE_DICT,
                BridgeError::MissingOSRef,
                BridgeError::InvalidOSRef,
            );
            let is_event = check_storage(
                original_storage_dict_ref,
                key,
                Option::None,
                Option::None,
                token_id.clone(),
                destination_chain.clone(),
                destination_user_address.clone(),
                source_nft_contract_address,
                metadata_uri.clone(),
            );
            if !is_event {
                // SEND MONEY TO SERVICE
                transfer_tx_fees(amount, sender_purse, Receiver::Service);
            }
            if is_event {
                transfer_tx_fees(amount, sender_purse, Receiver::Sender);
                casper_event_standard::emit(Locked::new(
                    token_id,
                    destination_chain,
                    destination_user_address,
                    source_nft_contract_address.to_string(),
                    U256::from(1),
                    nft_type,
                    self_chain,
                    metadata_uri,
                ));
            }
        }
    }
}
#[no_mangle]
pub extern "C" fn update_storage_and_process_lock() {
    // RUNTIME ARGS START
    let token_id: TokenIdentifier = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        BridgeError::MissingArgumentTokenId,
        BridgeError::InvalidArgumentTokenId,
    )
    .unwrap_or_revert();

    let destination_chain: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_CHAIN,
        BridgeError::MissingArgumentDestinationChain,
        BridgeError::InvalidArgumentDestinationChain,
    )
    .unwrap_or_revert();

    let destination_user_address: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_USER_ADDRESS,
        BridgeError::MissingArgumentDestinationUserAddress,
        BridgeError::InvalidArgumentDestinationUserAddress,
    )
    .unwrap_or_revert();

    let source_nft_contract_address: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_NFT_CONTRACT_ADDRESS,
        BridgeError::MissingArgumentSourceNftContractAddress,
        BridgeError::InvalidArgumentSourceNftContractAddress,
    )
    .unwrap_or_revert();

    let metadata_uri: String = utils::get_named_arg_with_user_errors(
        ARG_METADATA,
        BridgeError::MissingArgumentMetadata,
        BridgeError::InvalidArgumentMetadata,
    )
    .unwrap_or_revert();

    let storage_address_option: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_STORAGE_ADDRESS,
        BridgeError::MissingArgumentStorageAddress,
        BridgeError::InvalidArgumentStorageAddress,
    )
    .unwrap_or_revert();

    let nft_sender_address_option: AccountHash = utils::get_named_arg_with_user_errors(
        ARG_NFT_SENDER_ADDRESS,
        BridgeError::MissingArgumentNftSenderAddress,
        BridgeError::InvalidArgumentNftSenderAddress,
    )
    .unwrap_or_revert();
    // RUNTIME ARGS END

    // CHECKING IF CALLER IS SERVICE OR NOT START
    let service_account_ref = utils::get_uref(
        KEY_SERVICE_ADDRESS,
        BridgeError::MissingServiceAddressRef,
        BridgeError::InvalidServiceAddressRef,
    );
    let caller = runtime::get_caller();
    let service_account: CPublicKey = storage::read_or_revert(service_account_ref);
    if service_account.to_account_hash() != caller {
        runtime::revert(BridgeError::OnlyServiceCanCallThis);
    }
    // CHECKING IF CALLER IS SERVICE OR NOT END

    let self_chain_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );

    let self_chain: String = storage::read_or_revert(self_chain_ref);

    let nft_type_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );

    let nft_type: String = storage::read_or_revert(nft_type_ref);

    let duplicate_to_original_dict_ref = utils::get_uref(
        KEY_DUPLICATE_TO_ORIGINAL_DICT,
        BridgeError::MissingDTODictRef,
        BridgeError::InvalidOTDDictRef,
    );

    let mut data_key = source_nft_contract_address.to_bytes().unwrap();

    let self_chain_b = self_chain.to_bytes().unwrap();

    data_key.extend(self_chain_b);

    let mut hasher = Sha256::new();

    hasher.update(data_key);

    let hash = hasher.finalize();

    let binding = hex::encode(hash);

    let key = binding.as_str();

    let original_collection_address_option = storage::dictionary_get::<
        DuplicateToOriginalContractInfo,
    >(duplicate_to_original_dict_ref, key)
    .unwrap_or(None);

    match original_collection_address_option {
        Some(v) => {
            // notOriginal
            let duplicate_storage_dict_ref = utils::get_uref(
                KEY_DUPLICATE_STORAGE_DICT,
                BridgeError::MissingDSRef,
                BridgeError::InvalidDSRef,
            );
            let is_event = check_storage(
                duplicate_storage_dict_ref,
                key,
                Some(storage_address_option),
                Some(nft_sender_address_option),
                token_id.clone(),
                destination_chain.clone(),
                destination_user_address.clone(),
                source_nft_contract_address,
                metadata_uri.clone(),
            );
            if is_event {
                casper_event_standard::emit(Locked::new(
                    token_id,
                    destination_chain,
                    destination_user_address,
                    v.contract_address.to_string(),
                    U256::from(1),
                    nft_type,
                    v.chain,
                    metadata_uri,
                ));
            }
        }
        None => {
            // isOriginal
            let original_storage_dict_ref = utils::get_uref(
                KEY_ORIGINAL_STORAGE_DICT,
                BridgeError::MissingOSRef,
                BridgeError::InvalidOSRef,
            );
            let is_event = check_storage(
                original_storage_dict_ref,
                key,
                Some(storage_address_option),
                Some(nft_sender_address_option),
                token_id.clone(),
                destination_chain.clone(),
                destination_user_address.clone(),
                source_nft_contract_address,
                metadata_uri.clone(),
            );
            if is_event {
                casper_event_standard::emit(Locked::new(
                    token_id,
                    destination_chain,
                    destination_user_address,
                    source_nft_contract_address.to_string(),
                    U256::from(1),
                    nft_type,
                    self_chain,
                    metadata_uri,
                ));
            }
        }
    }
}
#[no_mangle]
pub extern "C" fn claim() {
    // RUNTIME ARGS START
    let token_id: String = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        BridgeError::MissingArgumentTokenId,
        BridgeError::InvalidArgumentTokenId,
    )
    .unwrap_or_revert();

    let source_chain: String = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_CHAIN,
        BridgeError::MissingArgumentSourceChain,
        BridgeError::InvalidArgumentSourceChain,
    )
    .unwrap_or_revert();

    let destination_chain: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_CHAIN,
        BridgeError::MissingArgumentDestinationChain,
        BridgeError::InvalidArgumentDestinationChain,
    )
    .unwrap_or_revert();

    let destination_user_address: AccountHash = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_USER_ADDRESS,
        BridgeError::MissingArgumentDestinationUserAddress,
        BridgeError::InvalidArgumentDestinationUserAddress,
    )
    .unwrap_or_revert();

    let source_nft_contract_address: String = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_NFT_CONTRACT_ADDRESS,
        BridgeError::MissingArgumentSourceNftContractAddress,
        BridgeError::InvalidArgumentSourceNftContractAddress,
    )
    .unwrap_or_revert();

    let name: String = utils::get_named_arg_with_user_errors(
        ARG_NAME,
        BridgeError::MissingArgumentName,
        BridgeError::InvalidArgumentName,
    )
    .unwrap_or_revert();

    let symbol: String = utils::get_named_arg_with_user_errors(
        ARG_SYMBOL,
        BridgeError::MissingArgumentSymbol,
        BridgeError::InvalidArgumentSymbol,
    )
    .unwrap_or_revert();

    let royalty: U512 = utils::get_named_arg_with_user_errors(
        ARG_ROYALTY,
        BridgeError::MissingArgumentRoyalty,
        BridgeError::InvalidArgumentRoyalty,
    )
    .unwrap_or_revert();

    let royalty_receiver: AccountHash = utils::get_named_arg_with_user_errors(
        ARG_ROYALTY_RECEIVER,
        BridgeError::MissingArgumentRoyaltyReceiver,
        BridgeError::InvalidArgumentRoyaltyReceiver,
    )
    .unwrap_or_revert();

    let metadata: String = utils::get_named_arg_with_user_errors(
        ARG_METADATA,
        BridgeError::MissingArgumentMetadata,
        BridgeError::InvalidArgumentMetadata,
    )
    .unwrap_or_revert();

    let transaction_hash: String = utils::get_named_arg_with_user_errors(
        ARG_TRANSACTION_HASH,
        BridgeError::MissingArgumentTransactionHash,
        BridgeError::InvalidArgumentTransactionHash,
    )
    .unwrap_or_revert();

    let token_amount: U512 = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_AMOUNT,
        BridgeError::MissingArgumentTokenAmount,
        BridgeError::InvalidArgumentTokenAmount,
    )
    .unwrap_or_revert();

    let nft_type: String = utils::get_named_arg_with_user_errors(
        ARG_NFT_TYPE,
        BridgeError::MissingArgumentNftType,
        BridgeError::InvalidArgumentNftType,
    )
    .unwrap_or_revert();

    let fee: U512 = utils::get_named_arg_with_user_errors(
        ARG_FEE,
        BridgeError::MissingArgumentFee,
        BridgeError::InvalidArgumentFee,
    )
    .unwrap_or_revert();

    let lock_tx_chain: String = utils::get_named_arg_with_user_errors(
        ARG_LOCK_TX_CHAIN,
        BridgeError::MissingArgumentLockTxChain,
        BridgeError::InvalidArgumentLockTxChain,
    )
    .unwrap_or_revert();

    let signatures: Vec<Sigs> = utils::get_named_arg_with_user_errors(
        ARG_SIGNATURES,
        BridgeError::MissingArgumentSignatures,
        BridgeError::InvalidArgumentSignatures,
    )
    .unwrap_or_revert();

    let sender_purse: URef = utils::get_named_arg_with_user_errors(
        ARG_SENDER_PURSE,
        BridgeError::MissingArgumentSenderPurse,
        BridgeError::InvalidArgumentSenderPurse,
    )
    .unwrap_or_revert();

    let amount: U512 = utils::get_named_arg_with_user_errors(
        ARG_AMOUNT,
        BridgeError::MissingArgumentAmount,
        BridgeError::InvalidArgumentAmount,
    )
    .unwrap_or_revert();
    // RUNTIME ARGS END

    let collection_deploy_fee_ref = utils::get_uref(
        KEY_COLLECTION_DEPLOY_FEE,
        BridgeError::MissingCollectionDeployFeeRef,
        BridgeError::InvalidCollectionDeployFeeRef,
    );

    let claim_fee_ref = utils::get_uref(
        KEY_CLAIM_FEE,
        BridgeError::MissingClaimFeeRef,
        BridgeError::InvalidClaimFeeRef,
    );

    let data = ClaimData {
        token_id,
        source_chain,
        destination_chain,
        destination_user_address,
        source_nft_contract_address,
        name,
        symbol,
        royalty,
        royalty_receiver,
        metadata,
        transaction_hash,
        token_amount,
        nft_type,
        fee,
        lock_tx_chain,
    };

    let collection_deploy_fee: U512 = storage::read_or_revert(collection_deploy_fee_ref);
    let claim_fee: U512 = storage::read_or_revert(claim_fee_ref);

    // CHECK COLLECTION DEPLOY FEE
    if amount < collection_deploy_fee {
        runtime::revert(BridgeError::CollectionFeeLessThanSentAmount);
    }

    let self_chain_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );

    let self_chain: String = storage::read_or_revert(self_chain_ref);

    let nft_type_ref = utils::get_uref(
        KEY_TYPE_ERC721,
        BridgeError::MissingTypeErc721Ref,
        BridgeError::InvalidTypeErc721Ref,
    );

    let self_nft_type: String = storage::read_or_revert(nft_type_ref);

    matches_current_chain(data.destination_chain.clone(), self_chain.clone());

    if data.nft_type != self_nft_type {
        runtime::revert(BridgeError::InvalidNftType);
    }

    let mut hasher = Sha256::new();

    let claim_data_vec = data
        .to_bytes()
        .unwrap_or_revert_with(BridgeError::ClaimDataConversionError);

    hasher.update(claim_data_vec);

    let hash = hasher.finalize();

    let binding = hex::encode(hash);

    let unique_key = binding.as_str();

    let unique_identifiers_dict_ref = utils::get_uref(
        KEY_UNIQUE_IDENTIFIERS_DICT,
        BridgeError::MissingUniqueIdentifiersDictRef,
        BridgeError::InvalidUniqueIdentifiersDictRef,
    );

    let mut unique_identifiers = (false, false);
    let unique_identifiers_option =
        storage::dictionary_get::<(bool, bool)>(unique_identifiers_dict_ref, unique_key)
            .unwrap_or(None);

    match unique_identifiers_option {
        Some(v) => {
            unique_identifiers = v;
        }
        None => {}
    }

    if unique_identifiers.0 && unique_identifiers.1 {
        runtime::revert(BridgeError::DataAlreadyProcessed);
    }
    if !unique_identifiers.0 && !unique_identifiers.1 {
        let validators_to_rewards = verify_signatures(data.to_bytes().unwrap(), signatures);
        transfer_tx_fees(claim_fee, sender_purse, Receiver::Bridge);
        reward_validators(data.fee, validators_to_rewards);
    }

    let original_to_duplicate_dict_ref = utils::get_uref(
        KEY_ORIGINAL_TO_DUPLICATE_DICT,
        BridgeError::MissingOTDDictRef,
        BridgeError::InvalidOTDDictRef,
    );

    let mut data_key = data.source_nft_contract_address.to_bytes().unwrap();

    let source_chain_b = data.source_chain.to_bytes().unwrap();

    data_key.extend(source_chain_b);

    let mut hasher = Sha256::new();

    hasher.update(data_key);

    let hash = hasher.finalize();

    let binding = hex::encode(hash);

    let otd_key = binding.as_str();

    let duplicate_collection_address_option = storage::dictionary_get::<
        OriginalToDuplicateContractInfo,
    >(original_to_duplicate_dict_ref, otd_key)
    .unwrap_or(None);

    let mut duplicate_collection_address = OriginalToDuplicateContractInfo {
        chain: "".to_string(),
        contract_address: Default::default(),
    };

    let mut has_duplicate: bool = false;
    let mut has_storage: bool = false;

    let storage_contract_option;
    let mut storage_contract: ContractHash = Default::default();

    match duplicate_collection_address_option {
        Some(v) => {
            duplicate_collection_address = v;
            let duplicate_storage_ref = utils::get_uref(
                KEY_DUPLICATE_STORAGE_DICT,
                BridgeError::MissingDSRef,
                BridgeError::InvalidDSRef,
            );

            let mut data_key = duplicate_collection_address
                .contract_address
                .to_bytes()
                .unwrap();

            let self_chain_b = self_chain.to_bytes().unwrap();

            data_key.extend(self_chain_b);

            let mut hasher = Sha256::new();

            hasher.update(data_key);

            let hash = hasher.finalize();

            let binding = hex::encode(hash);

            let ds_key = binding.as_str();

            storage_contract_option =
                storage::dictionary_get::<ContractHash>(duplicate_storage_ref, ds_key)
                    .unwrap_or(None);
            has_duplicate = true;
        }
        None => {
            let original_storage_ref = utils::get_uref(
                KEY_ORIGINAL_STORAGE_DICT,
                BridgeError::MissingOSRef,
                BridgeError::InvalidOSRef,
            );
            storage_contract_option =
                storage::dictionary_get::<ContractHash>(original_storage_ref, otd_key)
                    .unwrap_or(None);
        }
    }

    match storage_contract_option {
        Some(v) => {
            storage_contract = v;
            has_storage = true;
        }
        None => {}
    }

    if !has_duplicate && !has_storage {
        transfer_tx_fees(amount - claim_fee, sender_purse, Receiver::Service);
    } else {
        transfer_tx_fees(amount - claim_fee, sender_purse, Receiver::Sender);
    }

    // ===============================/ hasDuplicate && hasStorage /=======================
    if has_duplicate && has_storage {
        let nft_owner = owner_of(
            duplicate_collection_address.contract_address,
            TokenIdentifier::Hash(data.token_id.clone()),
        );

        if nft_owner == Key::from(storage_contract) {
            // UNLOCK NFT
            unlock_token(
                storage_contract,
                TokenIdentifier::Hash(data.token_id.clone()),
                data.destination_user_address,
            );
            storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));

            casper_event_standard::emit(UnLock::new(
                data.destination_user_address.to_string(),
                data.token_id.clone(),
                storage_contract.to_string(),
            ));
            casper_event_standard::emit(Claimed::new(
                data.lock_tx_chain,
                data.source_chain,
                data.transaction_hash,
                duplicate_collection_address.contract_address.to_string(),
                data.token_id,
            ));
        } else {
            // MINT NFT
            mint(
                duplicate_collection_address.contract_address,
                Key::from(data.destination_user_address),
                data.metadata,
            );
            storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));
            casper_event_standard::emit(Claimed::new(
                data.lock_tx_chain,
                data.source_chain,
                data.transaction_hash,
                duplicate_collection_address.contract_address.to_string(),
                data.token_id,
            ));
        }
    }
    // ===============================/ hasDuplicate && NOT hasStorage /=======================
    else if has_duplicate && !has_storage {
        // MINT NFT
        mint(
            duplicate_collection_address.contract_address,
            Key::from(data.destination_user_address),
            data.metadata,
        );
        storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));
        casper_event_standard::emit(Claimed::new(
            data.lock_tx_chain,
            data.source_chain,
            data.transaction_hash,
            duplicate_collection_address.contract_address.to_string(),
            data.token_id,
        ));
    }
    // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
    else if !has_duplicate && !has_storage {
        storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, false));
        casper_event_standard::emit(DeployCollection::new(
            TokenIdentifier::Hash(data.token_id),
            data.source_chain,
            data.destination_chain,
            data.destination_user_address,
            data.source_nft_contract_address,
            data.name,
            data.symbol,
            data.royalty,
            data.royalty_receiver,
            data.metadata,
            data.transaction_hash,
            data.token_amount,
            data.nft_type,
            data.fee,
            data.lock_tx_chain,
        ));
    }
    // ===============================/ NOT hasDuplicate && hasStorage /=======================
    else if !has_duplicate && has_storage {
        let contract =
            ContractHash::from_formatted_str(data.source_nft_contract_address.as_str()).unwrap();
        let nft_owner = owner_of(contract, TokenIdentifier::Hash(data.token_id.clone()));

        if nft_owner == Key::from(storage_contract) {
            // UNLOCK NFT
            unlock_token(
                storage_contract,
                TokenIdentifier::Hash(data.token_id.clone()),
                data.destination_user_address,
            );
            storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));
            casper_event_standard::emit(UnLock::new(
                data.destination_user_address.to_string(),
                data.token_id.clone(),
                storage_contract.to_string(),
            ));
            casper_event_standard::emit(Claimed::new(
                data.lock_tx_chain,
                data.source_chain,
                data.transaction_hash,
                data.source_nft_contract_address,
                data.token_id,
            ));
        } else {
            // CANT BE THERE
            runtime::revert(BridgeError::CantBeThere);
        }
    } else {
        runtime::revert(BridgeError::InvalidBridgeState);
    };
}
#[no_mangle]
pub extern "C" fn update_collection_process_claim() {
    // RUNTIME ARGS START
    let token_id: String = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        BridgeError::MissingArgumentTokenId,
        BridgeError::InvalidArgumentTokenId,
    )
    .unwrap_or_revert();

    let source_chain: String = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_CHAIN,
        BridgeError::MissingArgumentSourceChain,
        BridgeError::InvalidArgumentSourceChain,
    )
    .unwrap_or_revert();

    let destination_chain: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_CHAIN,
        BridgeError::MissingArgumentDestinationChain,
        BridgeError::InvalidArgumentDestinationChain,
    )
    .unwrap_or_revert();

    let destination_user_address: AccountHash = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_USER_ADDRESS,
        BridgeError::MissingArgumentDestinationUserAddress,
        BridgeError::InvalidArgumentDestinationUserAddress,
    )
    .unwrap_or_revert();

    let source_nft_contract_address: String = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_NFT_CONTRACT_ADDRESS,
        BridgeError::MissingArgumentSourceNftContractAddress,
        BridgeError::InvalidArgumentSourceNftContractAddress,
    )
    .unwrap_or_revert();

    let name: String = utils::get_named_arg_with_user_errors(
        ARG_NAME,
        BridgeError::MissingArgumentName,
        BridgeError::InvalidArgumentName,
    )
    .unwrap_or_revert();

    let symbol: String = utils::get_named_arg_with_user_errors(
        ARG_SYMBOL,
        BridgeError::MissingArgumentSymbol,
        BridgeError::InvalidArgumentSymbol,
    )
    .unwrap_or_revert();

    let royalty: U512 = utils::get_named_arg_with_user_errors(
        ARG_ROYALTY,
        BridgeError::MissingArgumentRoyalty,
        BridgeError::InvalidArgumentRoyalty,
    )
    .unwrap_or_revert();

    let royalty_receiver: AccountHash = utils::get_named_arg_with_user_errors(
        ARG_ROYALTY_RECEIVER,
        BridgeError::MissingArgumentRoyaltyReceiver,
        BridgeError::InvalidArgumentRoyaltyReceiver,
    )
    .unwrap_or_revert();

    let metadata: String = utils::get_named_arg_with_user_errors(
        ARG_METADATA,
        BridgeError::MissingArgumentMetadata,
        BridgeError::InvalidArgumentMetadata,
    )
    .unwrap_or_revert();

    let transaction_hash: String = utils::get_named_arg_with_user_errors(
        ARG_TRANSACTION_HASH,
        BridgeError::MissingArgumentTransactionHash,
        BridgeError::InvalidArgumentTransactionHash,
    )
    .unwrap_or_revert();

    let token_amount: U512 = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_AMOUNT,
        BridgeError::MissingArgumentTokenAmount,
        BridgeError::InvalidArgumentTokenAmount,
    )
    .unwrap_or_revert();

    let nft_type: String = utils::get_named_arg_with_user_errors(
        ARG_NFT_TYPE,
        BridgeError::MissingArgumentNftType,
        BridgeError::InvalidArgumentNftType,
    )
    .unwrap_or_revert();

    let fee: U512 = utils::get_named_arg_with_user_errors(
        ARG_FEE,
        BridgeError::MissingArgumentFee,
        BridgeError::InvalidArgumentFee,
    )
    .unwrap_or_revert();

    let lock_tx_chain: String = utils::get_named_arg_with_user_errors(
        ARG_LOCK_TX_CHAIN,
        BridgeError::MissingArgumentLockTxChain,
        BridgeError::InvalidArgumentLockTxChain,
    )
    .unwrap_or_revert();

    let collection_address: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_ADDRESS,
        BridgeError::MissingArgumentCollectionAddress,
        BridgeError::InvalidArgumentCollectionAddress,
    )
    .unwrap_or_revert();
    // RUNTIME ARGS END

    let data = ClaimData {
        token_id,
        source_chain,
        destination_chain,
        destination_user_address,
        source_nft_contract_address,
        name,
        symbol,
        royalty,
        royalty_receiver,
        metadata,
        transaction_hash,
        token_amount,
        nft_type,
        fee,
        lock_tx_chain,
    };

    // CHECKING IF CALLER IS SERVICE OR NOT START
    let service_account_ref = utils::get_uref(
        KEY_SERVICE_ADDRESS,
        BridgeError::MissingServiceAddressRef,
        BridgeError::InvalidServiceAddressRef,
    );
    let caller = runtime::get_caller();
    let service_account: CPublicKey = storage::read_or_revert(service_account_ref);
    if service_account.to_account_hash() != caller {
        runtime::revert(BridgeError::OnlyServiceCanCallThis);
    }
    // CHECKING IF CALLER IS SERVICE OR NOT END

    let self_chain_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );

    let self_chain: String = storage::read_or_revert(self_chain_ref);

    let nft_type_ref = utils::get_uref(
        KEY_TYPE_ERC721,
        BridgeError::MissingTypeErc721Ref,
        BridgeError::InvalidTypeErc721Ref,
    );

    let self_nft_type: String = storage::read_or_revert(nft_type_ref);

    matches_current_chain(data.destination_chain.clone(), self_chain.clone());

    if data.nft_type != self_nft_type {
        runtime::revert(BridgeError::InvalidNftType);
    }

    let mut hasher = Sha256::new();

    let claim_data_vec = data
        .to_bytes()
        .unwrap_or_revert_with(BridgeError::ClaimDataConversionError);

    hasher.update(claim_data_vec);

    let hash = hasher.finalize();

    let binding = hex::encode(hash);

    let unique_key = binding.as_str();

    let unique_identifiers_dict_ref = utils::get_uref(
        KEY_UNIQUE_IDENTIFIERS_DICT,
        BridgeError::MissingUniqueIdentifiersDictRef,
        BridgeError::InvalidUniqueIdentifiersDictRef,
    );

    let mut unique_identifiers = (false, false);
    let unique_identifiers_option =
        storage::dictionary_get::<(bool, bool)>(unique_identifiers_dict_ref, unique_key)
            .unwrap_or(None);

    match unique_identifiers_option {
        Some(v) => {
            unique_identifiers = v;
        }
        None => {}
    }

    if !unique_identifiers.0 {
        runtime::revert(BridgeError::NoClaimInfoFound)
    }
    if unique_identifiers.0 && unique_identifiers.1 {
        runtime::revert(BridgeError::DataAlreadyProcessed);
    }

    let original_to_duplicate_dict_ref = utils::get_uref(
        KEY_ORIGINAL_TO_DUPLICATE_DICT,
        BridgeError::MissingOTDDictRef,
        BridgeError::InvalidOTDDictRef,
    );

    let duplicate_to_original_dict_ref = utils::get_uref(
        KEY_DUPLICATE_TO_ORIGINAL_DICT,
        BridgeError::MissingDTODictRef,
        BridgeError::InvalidDTODictRef,
    );

    let mut data_key = data.source_nft_contract_address.to_bytes().unwrap();

    let source_chain_b = data.source_chain.to_bytes().unwrap();

    data_key.extend(source_chain_b);

    let mut hasher = Sha256::new();

    hasher.update(data_key);

    let hash = hasher.finalize();

    let binding = hex::encode(hash);

    let otd_key = binding.as_str();

    let duplicate_collection_address_option = storage::dictionary_get::<
        OriginalToDuplicateContractInfo,
    >(original_to_duplicate_dict_ref, otd_key)
    .unwrap_or(None);

    let mut duplicate_collection_address = OriginalToDuplicateContractInfo {
        chain: "".to_string(),
        contract_address: Default::default(),
    };

    let mut has_duplicate: bool = false;
    let mut has_storage: bool = false;

    let storage_contract_option;
    let mut storage_contract: ContractHash = Default::default();

    match duplicate_collection_address_option {
        Some(v) => {
            duplicate_collection_address = v;
            let duplicate_storage_ref = utils::get_uref(
                KEY_DUPLICATE_STORAGE_DICT,
                BridgeError::MissingDSRef,
                BridgeError::InvalidDSRef,
            );

            let mut data_key = duplicate_collection_address
                .contract_address
                .to_bytes()
                .unwrap();

            let self_chain_b = self_chain.to_bytes().unwrap();

            data_key.extend(self_chain_b);

            let mut hasher = Sha256::new();

            hasher.update(data_key);

            let hash = hasher.finalize();

            let binding = hex::encode(hash);

            let ds_key = binding.as_str();

            storage_contract_option =
                storage::dictionary_get::<ContractHash>(duplicate_storage_ref, ds_key)
                    .unwrap_or(None);
            has_duplicate = true;
        }
        None => {
            let original_storage_ref = utils::get_uref(
                KEY_ORIGINAL_STORAGE_DICT,
                BridgeError::MissingOSRef,
                BridgeError::InvalidOSRef,
            );

            storage_contract_option =
                storage::dictionary_get::<ContractHash>(original_storage_ref, otd_key)
                    .unwrap_or(None);
        }
    }

    match storage_contract_option {
        Some(v) => {
            storage_contract = v;
            has_storage = true;
        }
        None => {}
    }

    // ===============================/ hasDuplicate && hasStorage /=======================
    if has_duplicate && has_storage {
        let nft_owner = owner_of(
            duplicate_collection_address.contract_address,
            TokenIdentifier::Hash(data.token_id.clone()),
        );

        if nft_owner == Key::from(storage_contract) {
            // UNLOCK NFT
            unlock_token(
                storage_contract,
                TokenIdentifier::Hash(data.token_id.clone()),
                data.destination_user_address,
            );
            storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));

            casper_event_standard::emit(UnLock::new(
                data.destination_user_address.to_string(),
                data.token_id.clone(),
                storage_contract.to_string(),
            ));
            casper_event_standard::emit(Claimed::new(
                data.lock_tx_chain,
                data.source_chain,
                data.transaction_hash,
                duplicate_collection_address.contract_address.to_string(),
                data.token_id,
            ));
        } else {
            // MINT NFT
            mint(
                duplicate_collection_address.contract_address,
                Key::from(data.destination_user_address),
                data.metadata,
            );
            storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));
            casper_event_standard::emit(Claimed::new(
                data.lock_tx_chain,
                data.source_chain,
                data.transaction_hash,
                duplicate_collection_address.contract_address.to_string(),
                data.token_id,
            ));
        }
    }
    // ===============================/ hasDuplicate && NOT hasStorage /=======================
    else if has_duplicate && !has_storage {
        // MINT NFT
        mint(
            duplicate_collection_address.contract_address,
            Key::from(data.destination_user_address),
            data.metadata,
        );
        storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));
        casper_event_standard::emit(Claimed::new(
            data.lock_tx_chain,
            data.source_chain,
            data.transaction_hash,
            duplicate_collection_address.contract_address.to_string(),
            data.token_id,
        ));
    }
    // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
    else if !has_duplicate && !has_storage {
        storage::dictionary_put(
            original_to_duplicate_dict_ref,
            otd_key,
            OriginalToDuplicateContractInfo {
                chain: self_chain.clone(),
                contract_address: collection_address,
            },
        );

        let mut data_key = collection_address.to_bytes().unwrap();

        let self_chain_b = self_chain.clone().to_bytes().unwrap();

        data_key.extend(self_chain_b);

        let mut hasher = Sha256::new();

        hasher.update(data_key);

        let hash = hasher.finalize();

        let binding = hex::encode(hash);

        let dto_key = binding.as_str();

        storage::dictionary_put(
            duplicate_to_original_dict_ref,
            dto_key,
            DuplicateToOriginalContractInfo {
                chain: data.source_chain.clone(),
                contract_address: data.source_nft_contract_address,
            },
        );

        // MINT NFT
        mint(
            collection_address,
            Key::from(data.destination_user_address),
            data.metadata,
        );
        storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));
        casper_event_standard::emit(Claimed::new(
            data.lock_tx_chain,
            data.source_chain,
            data.transaction_hash,
            collection_address.to_string(),
            data.token_id,
        ));
    }
    // ===============================/ NOT hasDuplicate && hasStorage /=======================
    else if !has_duplicate && has_storage {
        let contract =
            ContractHash::from_formatted_str(data.source_nft_contract_address.as_str()).unwrap();
        let nft_owner = owner_of(contract, TokenIdentifier::Hash(data.token_id.clone()));

        if nft_owner == Key::from(storage_contract) {
            // UNLOCK NFT
            unlock_token(
                storage_contract,
                TokenIdentifier::Hash(data.token_id.clone()),
                data.destination_user_address,
            );
            storage::dictionary_put(unique_identifiers_dict_ref, unique_key, (true, true));
            casper_event_standard::emit(UnLock::new(
                data.destination_user_address.to_string(),
                data.token_id.clone(),
                storage_contract.to_string(),
            ));
            casper_event_standard::emit(Claimed::new(
                data.lock_tx_chain,
                data.source_chain,
                data.transaction_hash,
                data.source_nft_contract_address,
                data.token_id,
            ));
        } else {
            // CANT BE THERE
            runtime::revert(BridgeError::CantBeThere);
        }
    } else {
        runtime::revert(BridgeError::InvalidBridgeState);
    };
}
fn generate_entry_points() -> EntryPoints {
    let mut entrypoints = EntryPoints::new();

    let init = EntryPoint::new(
        ENTRY_POINT_BRIDGE_INITIALIZE,
        vec![
            Parameter::new(ARG_VALIDATOR, CLType::PublicKey),
            Parameter::new(ARG_CHAIN_TYPE, CLType::String),
            Parameter::new(ARG_COLLECTION_DEPLOYER, CLType::ByteArray(32)),
            Parameter::new(ARG_STORAGE_DEPLOYER, CLType::ByteArray(32)),
            Parameter::new(ARG_SERVICE_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_SELF_HASH, CLType::ByteArray(32)),
            Parameter::new(ARG_STORAGE_DEPLOY_FEE, CLType::U512),
            Parameter::new(ARG_COLLECTION_DEPLOY_FEE, CLType::U512),
            Parameter::new(ARG_CLAIM_FEE, CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let add_validator = EntryPoint::new(
        ENTRY_POINT_BRIDGE_ADD_VALIDATOR,
        vec![
            Parameter::new(ARG_NEW_VALIDATOR_PUBLIC_KEY, CLType::ByteArray(32)),
            Parameter::new(
                ARG_SIGNATURES,
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::PublicKey),
                    Box::new(CLType::ByteArray(64)),
                ]))),
            ),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let blacklist_validator = EntryPoint::new(
        ENTRY_POINT_BRIDGE_BLACKLIST_VALIDATOR,
        vec![
            Parameter::new(ARG_VALIDATOR_PUBLIC_KEY, CLType::ByteArray(32)),
            Parameter::new(
                ARG_SIGNATURES,
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::PublicKey),
                    Box::new(CLType::ByteArray(64)),
                ]))),
            ),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let claim_reward_validator = EntryPoint::new(
        ENTRY_POINT_BRIDGE_CLAIM_REWARD_VALIDATOR,
        vec![Parameter::new(
            ARG_VALIDATOR_PUBLIC_KEY,
            CLType::ByteArray(32),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let lock = EntryPoint::new(
        ENTRY_POINT_BRIDGE_LOCK,
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_DESTINATION_CHAIN, CLType::String),
            Parameter::new(ARG_DESTINATION_USER_ADDRESS, CLType::String),
            Parameter::new(ARG_SOURCE_NFT_CONTRACT_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_METADATA, CLType::String),
            Parameter::new(ARG_SENDER_PURSE, CLType::URef),
            Parameter::new(ARG_AMOUNT, CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let update_storage_and_process_lock = EntryPoint::new(
        ENTRY_POINT_UPDATE_STORAGE_AND_PROCESS_LOCK,
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_DESTINATION_CHAIN, CLType::String),
            Parameter::new(ARG_DESTINATION_USER_ADDRESS, CLType::String),
            Parameter::new(ARG_SOURCE_NFT_CONTRACT_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_METADATA, CLType::String),
            Parameter::new(ARG_STORAGE_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_NFT_SENDER_ADDRESS, CLType::ByteArray(32)),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let claim = EntryPoint::new(
        ENTRY_POINT_BRIDGE_CLAIM,
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_SOURCE_CHAIN, CLType::String),
            Parameter::new(ARG_DESTINATION_CHAIN, CLType::String),
            Parameter::new(ARG_DESTINATION_USER_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_SOURCE_NFT_CONTRACT_ADDRESS, CLType::String),
            Parameter::new(ARG_NAME, CLType::String),
            Parameter::new(ARG_SYMBOL, CLType::String),
            Parameter::new(ARG_ROYALTY, CLType::U512),
            Parameter::new(ARG_ROYALTY_RECEIVER, CLType::ByteArray(32)),
            Parameter::new(ARG_METADATA, CLType::String),
            Parameter::new(ARG_TRANSACTION_HASH, CLType::String),
            Parameter::new(ARG_TOKEN_AMOUNT, CLType::U512),
            Parameter::new(ARG_NFT_TYPE, CLType::String),
            Parameter::new(ARG_FEE, CLType::U512),
            Parameter::new(ARG_LOCK_TX_CHAIN, CLType::String),
            Parameter::new(
                ARG_SIGNATURES,
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::PublicKey),
                    Box::new(CLType::ByteArray(64)),
                ]))),
            ),
            Parameter::new(ARG_SENDER_PURSE, CLType::URef),
            Parameter::new(ARG_AMOUNT, CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let update_collection_process_claim = EntryPoint::new(
        ENTRY_POINT_UPDATE_COLLECTION_AND_PROCESS_LOCK,
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_SOURCE_CHAIN, CLType::String),
            Parameter::new(ARG_DESTINATION_CHAIN, CLType::String),
            Parameter::new(ARG_DESTINATION_USER_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_SOURCE_NFT_CONTRACT_ADDRESS, CLType::String),
            Parameter::new(ARG_NAME, CLType::String),
            Parameter::new(ARG_SYMBOL, CLType::String),
            Parameter::new(ARG_ROYALTY, CLType::U512),
            Parameter::new(ARG_ROYALTY_RECEIVER, CLType::ByteArray(32)),
            Parameter::new(ARG_METADATA, CLType::String),
            Parameter::new(ARG_TRANSACTION_HASH, CLType::String),
            Parameter::new(ARG_TOKEN_AMOUNT, CLType::U512),
            Parameter::new(ARG_NFT_TYPE, CLType::String),
            Parameter::new(ARG_FEE, CLType::U512),
            Parameter::new(ARG_LOCK_TX_CHAIN, CLType::String),
            Parameter::new(ARG_COLLECTION_ADDRESS, CLType::ByteArray(32)),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entrypoints.add_entry_point(init);
    entrypoints.add_entry_point(add_validator);
    entrypoints.add_entry_point(blacklist_validator);
    entrypoints.add_entry_point(claim_reward_validator);
    entrypoints.add_entry_point(lock);
    entrypoints.add_entry_point(update_storage_and_process_lock);
    entrypoints.add_entry_point(claim);
    entrypoints.add_entry_point(update_collection_process_claim);
    entrypoints
}
fn install_bridge() {
    let entry_points = generate_entry_points();
    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());

        named_keys
    };

    let (contract_hash, _) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(HASH_KEY_NAME.to_string()),
        Some(ACCESS_KEY_NAME.to_string()),
    );

    let num: U512 = runtime::get_named_arg("number");

    runtime::put_key(
        &(THIS_CONTRACT.to_string() + &num.to_string()),
        contract_hash.into(),
    );
}
#[no_mangle]
pub extern "C" fn call() {
    install_bridge();
}
