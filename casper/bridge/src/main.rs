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
    BlackListValidator, Claimed, CollectionDeployFee, DeployCollection, DeployStorage, Locked,
    RewardValidator, StorageDeployFee,
};
use crate::structs::{
    ClaimData, DataType, DoneInfo, DuplicateToOriginalContractInfo,
    OriginalToDuplicateContractInfo, OtherToken, Receiver, SelfToken, Sigs, Waiting,
};
use crate::utils::encode_dictionary_item_key;
use casper_types::account::AccountHash;
use casper_types::bytesrepr::FromBytes;
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
use sha2::{Digest, Sha256};
use structs::Validator;

fn get_service_acc_hash() -> AccountHash {
    let service_ref = utils::get_uref(
        KEY_SERVICE_ADDRESS,
        BridgeError::MissingServiceAddressRef,
        BridgeError::InvalidServiceAddressRef,
    );
    let receiver_acc_hash: CPublicKey = storage::read_or_revert(service_ref);
    receiver_acc_hash.to_account_hash()
}
fn transfer_tx_fees(amount: U512, sender_purse: URef, to: Receiver) {
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
fn save_done_info(data: (Vec<Sigs>, DoneInfo), ss_key: String) {
    let submitted_signatures_dict_ref = utils::get_uref(
        KEY_SUBMITTED_SIGNATURES_DICT,
        BridgeError::MissingSubmittedSignaturesDictRef,
        BridgeError::InvalidSubmittedSignaturesDictRef,
    );

    storage::dictionary_put(submitted_signatures_dict_ref, ss_key.as_str(), data);
}
fn get_done_info(key: String) -> (Vec<Sigs>, DoneInfo) {
    let submitted_signatures_dict_ref = utils::get_uref(
        KEY_SUBMITTED_SIGNATURES_DICT,
        BridgeError::MissingSubmittedSignaturesDictRef,
        BridgeError::InvalidSubmittedSignaturesDictRef,
    );

    let submitted_signatures_data = storage::dictionary_get::<(Vec<Sigs>, DoneInfo)>(
        submitted_signatures_dict_ref,
        key.as_str(),
    )
    .unwrap_or_revert_with(BridgeError::NoSignaturesSubmitted)
    .unwrap_or_revert_with(BridgeError::NoSignaturesSubmitted);

    (submitted_signatures_data.0, submitted_signatures_data.1)
}
fn submit_sigs(data: Vec<u8>, data_type: DataType, signatures: Vec<Sigs>) {
    let key = hex::encode(data.clone());
    let submitted_signatures_dict_ref = utils::get_uref(
        KEY_SUBMITTED_SIGNATURES_DICT,
        BridgeError::MissingSubmittedSignaturesDictRef,
        BridgeError::InvalidSubmittedSignaturesDictRef,
    );

    let existing_signatures_option = storage::dictionary_get::<(Vec<Sigs>, DoneInfo)>(
        submitted_signatures_dict_ref,
        key.as_str(),
    )
    .unwrap_or(None);

    verify_signatures(
        data,
        data_type,
        signatures,
        existing_signatures_option,
        submitted_signatures_dict_ref,
        key.as_str(),
    );
}
fn create_hash_key(data: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let binding = hex::encode(hash);
    let key = binding.as_str();
    key.to_string()
}
fn verify_signatures(
    data: Vec<u8>,
    data_type: DataType,
    signatures: Vec<Sigs>,
    existing_signatures_option: Option<(Vec<Sigs>, DoneInfo)>,
    submitted_signatures_dict_ref: URef,
    ss_key: &str,
) {
    let mut percentage: u64 = 0;

    let mut uv: Vec<CPublicKey> = [].to_vec();
    let mut valid_signatures: Vec<Sigs> = vec![];

    let mut done_info = DoneInfo {
        done: false,
        can_do: false,
        fee_taken: false,
        data_type,
    };
    match existing_signatures_option {
        Some(v) => {
            done_info = v.1;
            valid_signatures = v.0.clone();
            percentage = v.0.len() as u64;
        }
        None => {}
    }

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );

    // HASH FOR SIGNATURE VERIFICATION START
    // let mut hasher = Sha512::new();
    // hasher.update(data);
    // let hash = hasher.finalize();
    // HASH FOR SIGNATURE VERIFICATION END

    for arg in signatures.into_iter() {
        let key = arg
            .public_key
            .to_bytes()
            .unwrap()
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<u8>>();
        let sig = arg.signature.to_bytes().unwrap();

        let signature = Signature::new(
            sig.as_slice()
                .try_into()
                .map_err(|_| BridgeError::FailedToPrepareSignature)
                .unwrap_or_revert(),
        );

        let key = EPublicKey::from_bytes(key.as_slice())
            .map_err(|_| BridgeError::FailedToPreparePublicKey)
            .unwrap_or_revert();

        let valid = key.verify(&data, &signature);

        if valid.is_ok() {
            let validator_key =
                encode_dictionary_item_key(Key::from(arg.public_key.to_account_hash()));
            let validator_exists =
                storage::dictionary_get::<Validator>(validators_dict_ref, validator_key.as_str())
                    .unwrap_or(None);

            match validator_exists {
                Some(v) => {
                    if v.added && !uv.contains(&arg.public_key) {
                        if !valid_signatures.contains(&arg) {
                            percentage = percentage + 1;
                            valid_signatures.push(arg.clone());
                        }
                        uv.push(arg.public_key);
                    }
                }
                None => {
                    // runtime::revert(BridgeError::Hello);
                }
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
        done_info.can_do = true;
        storage::dictionary_put(
            submitted_signatures_dict_ref,
            ss_key,
            (valid_signatures, done_info),
        );
    } else {
        done_info.can_do = false;
        storage::dictionary_put(
            submitted_signatures_dict_ref,
            ss_key,
            (valid_signatures, done_info),
        );
    }
}
fn check_storage(
    storage_dict_ref: URef,
    key: &str,
    token_id: TokenIdentifier,
    source_nft_contract_address: ContractHash,
) -> bool {
    let storage_address_option =
        storage::dictionary_get::<ContractHash>(storage_dict_ref, key).unwrap_or(None);
    match storage_address_option {
        Some(v) => {
            // TRANSFER TO STORAGE
            transfer_to_storage(v, source_nft_contract_address, token_id);
            true
        }
        None => {
            // EMIT DEPLOY STORAGE EVENT
            let waiting_dict_ref = utils::get_uref(
                KEY_WAITING_DICT,
                BridgeError::MissingWaitingDictRef,
                BridgeError::InvalidWaitingDictRef,
            );

            storage::dictionary_put(
                waiting_dict_ref,
                key,
                Waiting {
                    wait: true,
                    data_type: "storage".to_string(),
                },
            );

            casper_event_standard::emit(DeployStorage::new(
                source_nft_contract_address.to_string(),
            ));
            false
        }
    }
}
fn transfer_to_storage(
    storage_address: ContractHash,
    source_nft_contract_address: ContractHash,
    token_id: TokenIdentifier,
) {
    let from_address = Key::from(runtime::get_caller());

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

    let fee_per_validator = fee / validators.len();

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );

    for arg in validators {
        let validator_key = encode_dictionary_item_key(Key::from(arg.to_account_hash()));
        let validator_option =
            storage::dictionary_get::<Validator>(validators_dict_ref, validator_key.as_str())
                .unwrap_or_revert_with(BridgeError::FailedToGetValidatorForReward);
        match validator_option {
            Some(mut v) => {
                v.pending_rewards += fee_per_validator;
                storage::dictionary_put(validators_dict_ref, validator_key.as_str(), v);
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
    let service_account: CPublicKey = runtime::get_named_arg(ARG_SERVICE_ADDRESS);
    let self_hash: ContractHash = runtime::get_named_arg(ARG_SELF_HASH);

    let storage_deploy_fee: U512 = runtime::get_named_arg(ARG_STORAGE_DEPLOY_FEE);
    let collection_deploy_fee: U512 = runtime::get_named_arg(ARG_COLLECTION_DEPLOY_FEE);

    runtime::put_key(INITIALIZED, storage::new_uref(true).into());
    runtime::put_key(KEY_CHAIN_TYPE, storage::new_uref(chain_type).into());
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

    // INITIALIZING BOOTSTRAP VALIDATOR
    let validators_dict = storage::new_dictionary(KEY_VALIDATORS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    let validator_key = encode_dictionary_item_key(Key::from(validator.to_account_hash()));

    storage::dictionary_put(
        validators_dict,
        validator_key.as_str(),
        Validator {
            added: true,
            pending_rewards: U512::from(0),
        },
    );

    // INITIALIZING VALIDATOR COUNT
    runtime::put_key(KEY_VALIDATORS_COUNT, storage::new_uref(1u64).into());

    // INITIALIZING STORAGE DEPLOY FEE NONCE
    runtime::put_key(KEY_STORAGE_DEPLOY_FEE_NONCE, storage::new_uref(0u64).into());

    // INITIALIZING COLLECTION DEPLOY FEE NONCE
    runtime::put_key(
        KEY_COLLECTION_DEPLOY_FEE_NONCE,
        storage::new_uref(0u64).into(),
    );

    // INITIALIZING BLACKLIST VALIDATORS DICT
    storage::new_dictionary(KEY_BLACKLIST_VALIDATORS_DICT)
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

    // INITIALIZING SUBMITTED SIGNATURES DICT
    storage::new_dictionary(KEY_SUBMITTED_SIGNATURES_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING TOKEN INFO GET SELF DICT
    storage::new_dictionary(KEY_TOKEN_INFO_GET_SELF_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING TOKEN INFO KEY SELF DICT
    storage::new_dictionary(KEY_TOKEN_INFO_KEY_SELF_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    // INITIALIZING WAITING DICT
    storage::new_dictionary(KEY_WAITING_DICT)
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

    let blacklist_exists = storage::dictionary_get::<bool>(
        blacklist_validators_dict_ref,
        &new_validator_public_key.to_string(),
    )
    .unwrap_or(Option::from(false));

    match blacklist_exists {
        Some(_) => {
            runtime::revert(BridgeError::AlreadyBlacklisted);
        }
        None => {}
    }

    let data = new_validator_public_key
        .to_bytes()
        .unwrap()
        .iter()
        .skip(1)
        .cloned()
        .collect::<Vec<u8>>();

    let key = create_hash_key(data);
    let mut submitted_sigs_data = get_done_info(key.clone());
    let done_info = submitted_sigs_data.clone().1;

    if done_info.data_type != DataType::AddValidator {
        runtime::revert(BridgeError::InvalidDataType);
    }
    if done_info.done {
        runtime::revert(BridgeError::ValidatorAlreadyExists);
    }

    if done_info.can_do {
        let validators_count_ref = utils::get_uref(
            KEY_VALIDATORS_COUNT,
            BridgeError::MissingValidatorsCountRef,
            BridgeError::InvalidValidatorsCountRef,
        );
        let validators_count: u64 = storage::read_or_revert(validators_count_ref);

        let validator_key =
            encode_dictionary_item_key(Key::from(new_validator_public_key.to_account_hash()));
        storage::dictionary_put(
            validators_dict_ref,
            validator_key.as_str(),
            Validator {
                added: true,
                pending_rewards: U512::from(0),
            },
        );

        storage::write(validators_count_ref, validators_count + 1);

        submitted_sigs_data.1.done = true;
        save_done_info(submitted_sigs_data, key);

        casper_event_standard::emit(AddNewValidator::new(new_validator_public_key));
    } else {
        runtime::revert(BridgeError::MoreSignaturesNeeded);
    }
}
#[no_mangle]
pub extern "C" fn blacklist_validator() {
    let validator_public_key: CPublicKey = utils::get_named_arg_with_user_errors(
        ARG_VALIDATOR_PUBLIC_KEY,
        BridgeError::MissingArgumentValidator,
        BridgeError::InvalidArgumentValidator,
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
    let validator_key =
        encode_dictionary_item_key(Key::from(validator_public_key.to_account_hash()));
    storage::dictionary_get::<Validator>(validators_dict_ref, validator_key.as_str())
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

    let mut data = validator_public_key
        .to_bytes()
        .unwrap()
        .iter()
        .skip(1)
        .cloned()
        .collect::<Vec<u8>>();

    let black_list_vec = "blacklist".to_bytes().unwrap();

    data.extend(black_list_vec);

    let key = create_hash_key(data);
    let mut submitted_sigs_data = get_done_info(key.clone());
    let done_info = submitted_sigs_data.clone().1;

    if done_info.data_type != DataType::BlacklistValidator {
        runtime::revert(BridgeError::InvalidDataType);
    }
    if done_info.done {
        runtime::revert(BridgeError::AlreadyBlacklisted);
    }

    if done_info.can_do {
        let validators_count_ref = utils::get_uref(
            KEY_VALIDATORS_COUNT,
            BridgeError::MissingValidatorsCountRef,
            BridgeError::InvalidValidatorsCountRef,
        );
        let validators_count: u64 = storage::read_or_revert(validators_count_ref);

        let validator_key =
            encode_dictionary_item_key(Key::from(validator_public_key.to_account_hash()));

        storage::dictionary_put(
            validators_dict_ref,
            validator_key.as_str(),
            Option::<String>::None,
        );

        storage::dictionary_put(blacklist_validators_dict_ref, validator_key.as_str(), true);
        storage::write(validators_count_ref, validators_count - 1);

        submitted_sigs_data.1.done = true;
        save_done_info(submitted_sigs_data, key);

        casper_event_standard::emit(BlackListValidator::new(validator_public_key));
    } else {
        runtime::revert(BridgeError::MoreSignaturesNeeded);
    }
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
    let validator_key =
        encode_dictionary_item_key(Key::from(validator_public_key.to_account_hash()));
    let validator_option =
        storage::dictionary_get::<Validator>(validators_dict_ref, validator_key.as_str())
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
                        let validator_key = encode_dictionary_item_key(Key::from(
                            validator_public_key.to_account_hash(),
                        ));
                        storage::dictionary_put(
                            validators_dict_ref,
                            validator_key.as_str(),
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
pub extern "C" fn change_collection_deploy_fee() {
    let collection_deploy_fee: U512 = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_DEPLOY_FEE,
        BridgeError::MissingArgumentCollectionDeployFee,
        BridgeError::InvalidArgumentCollectionDeployFee,
    )
    .unwrap_or_revert();

    let collection_deploy_fee_nonce_ref = utils::get_uref(
        KEY_COLLECTION_DEPLOY_FEE_NONCE,
        BridgeError::MissingCollectionDeployFeeNonceRef,
        BridgeError::InvalidCollectionDeployFeeNonceRef,
    );

    let collection_deploy_fee_nonce: u64 = storage::read_or_revert(collection_deploy_fee_nonce_ref);

    let mut data = collection_deploy_fee.to_bytes().unwrap();
    data.extend(collection_deploy_fee_nonce.to_bytes().unwrap());

    let key = create_hash_key(data);
    let mut submitted_sigs_data = get_done_info(key.clone());
    let done_info = submitted_sigs_data.clone().1;

    if done_info.data_type != DataType::ChangeCollectionDeployFee {
        runtime::revert(BridgeError::InvalidDataType);
    }
    if done_info.done {
        runtime::revert(BridgeError::AlreadyExists);
    }

    if done_info.can_do {
        let collection_deploy_fee_ref = utils::get_uref(
            KEY_COLLECTION_DEPLOY_FEE,
            BridgeError::MissingCollectionDeployFeeRef,
            BridgeError::InvalidCollectionDeployFeeRef,
        );
        storage::write(collection_deploy_fee_ref, collection_deploy_fee);
        storage::write(
            collection_deploy_fee_nonce_ref,
            collection_deploy_fee_nonce + 1,
        );

        submitted_sigs_data.1.done = true;
        save_done_info(submitted_sigs_data, key);

        casper_event_standard::emit(CollectionDeployFee::new(collection_deploy_fee));
    } else {
        runtime::revert(BridgeError::MoreSignaturesNeeded);
    }
}
#[no_mangle]
pub extern "C" fn change_storage_deploy_fee() {
    let storage_deploy_fee: U512 = utils::get_named_arg_with_user_errors(
        ARG_STORAGE_DEPLOY_FEE,
        BridgeError::MissingArgumentStorageDeployFee,
        BridgeError::InvalidArgumentStorageDeployFee,
    )
    .unwrap_or_revert();

    let storage_deploy_fee_nonce_ref = utils::get_uref(
        KEY_STORAGE_DEPLOY_FEE_NONCE,
        BridgeError::MissingStorageDeployFeeNonceRef,
        BridgeError::InvalidStorageDeployFeeNonceRef,
    );

    let storage_deploy_fee_nonce: u64 = storage::read_or_revert(storage_deploy_fee_nonce_ref);

    let mut data = storage_deploy_fee.to_bytes().unwrap();
    data.extend(storage_deploy_fee_nonce.to_bytes().unwrap());

    let key = create_hash_key(data);
    let mut submitted_sigs_data = get_done_info(key.clone());
    let done_info = submitted_sigs_data.clone().1;

    if done_info.data_type != DataType::ChangeStorageDeployFee {
        runtime::revert(BridgeError::InvalidDataType);
    }
    if done_info.done {
        runtime::revert(BridgeError::AlreadyExists);
    }

    if done_info.can_do {
        let storage_deploy_fee_ref = utils::get_uref(
            KEY_STORAGE_DEPLOY_FEE,
            BridgeError::MissingStorageDeployFeeRef,
            BridgeError::InvalidStorageDeployFeeRef,
        );
        storage::write(storage_deploy_fee_ref, storage_deploy_fee);
        storage::write(storage_deploy_fee_nonce_ref, storage_deploy_fee_nonce + 1);

        submitted_sigs_data.1.done = true;
        save_done_info(submitted_sigs_data, key);

        casper_event_standard::emit(StorageDeployFee::new(storage_deploy_fee));
    } else {
        runtime::revert(BridgeError::MoreSignaturesNeeded);
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
        KEY_TYPE_ERC721,
        BridgeError::MissingTypeErc721Ref,
        BridgeError::InvalidTypeErc721Ref,
    );

    let nft_type: String = storage::read_or_revert(nft_type_ref);

    let duplicate_to_original_dict_ref = utils::get_uref(
        KEY_DUPLICATE_TO_ORIGINAL_DICT,
        BridgeError::MissingDTODictRef,
        BridgeError::InvalidOTDDictRef,
    );

    let self_chain_vec = self_chain.to_bytes().unwrap();
    let source_nft_contract_address_vec = source_nft_contract_address.to_bytes().unwrap();

    let mut mut_source_nft_contract_address_vec = source_nft_contract_address_vec.clone();
    mut_source_nft_contract_address_vec.extend(self_chain_vec.clone());

    let key = create_hash_key(mut_source_nft_contract_address_vec);

    let waiting_dict_ref = utils::get_uref(
        KEY_WAITING_DICT,
        BridgeError::MissingWaitingDictRef,
        BridgeError::InvalidWaitingDictRef,
    );

    let waiting =
        storage::dictionary_get::<Waiting>(waiting_dict_ref, key.as_str()).unwrap_or(None);

    match waiting {
        Some(v) => {
            if v.wait && v.data_type == "storage" {
                runtime::revert(BridgeError::WaitingForStorage);
            }
        }
        None => {}
    }

    let original_collection_address_option = storage::dictionary_get::<
        DuplicateToOriginalContractInfo,
    >(duplicate_to_original_dict_ref, key.as_str())
    .unwrap_or(None);

    // TOKEN

    let token_info_key_self_dict_ref = utils::get_uref(
        KEY_TOKEN_INFO_KEY_SELF_DICT,
        BridgeError::MissingTokenInfoKeySelfDictRef,
        BridgeError::InvalidTokenInfoKeySelfDictRef,
    );

    let token_info_get_self_dict_ref = utils::get_uref(
        KEY_TOKEN_INFO_GET_SELF_DICT,
        BridgeError::MissingTokenInfoGetSelfDictRef,
        BridgeError::InvalidTokenInfoGetSelfDictRef,
    );

    // 0 casper 0a234 ->  100 bsc 0x123
    let mut mut_token_id_vec = token_id.to_bytes().unwrap();
    mut_token_id_vec.extend(self_chain_vec.clone());
    mut_token_id_vec.extend(source_nft_contract_address_vec.clone());

    let token_key = create_hash_key(mut_token_id_vec);

    let token_info_key_self_option =
        storage::dictionary_get::<OtherToken>(token_info_key_self_dict_ref, token_key.as_str())
            .unwrap_or(None);

    let mut mut_token_id: Vec<u8> = vec![];

    match token_info_key_self_option {
        Some(v) => {
            mut_token_id = v.token_id.to_bytes().unwrap();
        }
        None => {
            mut_token_id = token_id.to_bytes().unwrap();

            storage::dictionary_put(
                token_info_key_self_dict_ref,
                token_key.as_str(),
                OtherToken {
                    token_id: token_id.to_string(),
                    chain: self_chain.clone(),
                    contract_address: source_nft_contract_address.to_string(),
                },
            );

            storage::dictionary_put(
                token_info_get_self_dict_ref,
                token_key.as_str(),
                SelfToken {
                    token_id: token_id.clone(),
                    chain: self_chain.clone(),
                    contract_address: source_nft_contract_address,
                },
            );
        }
    }
    let final_token_id = TokenIdentifier::from_bytes(&*mut_token_id).unwrap().0;
    // TOKEN

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
                key.as_str(),
                token_id,
                source_nft_contract_address,
            );
            if !is_event {
                // SEND MONEY TO SERVICE
                transfer_tx_fees(amount, sender_purse, Receiver::Service);
            }

            if is_event {
                transfer_tx_fees(amount, sender_purse, Receiver::Sender);
                casper_event_standard::emit(Locked::new(
                    final_token_id,
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
                key.as_str(),
                token_id,
                source_nft_contract_address,
            );
            if !is_event {
                // SEND MONEY TO SERVICE
                transfer_tx_fees(amount, sender_purse, Receiver::Service);
            }
            if is_event {
                transfer_tx_fees(amount, sender_purse, Receiver::Sender);
                casper_event_standard::emit(Locked::new(
                    final_token_id,
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
pub extern "C" fn update_storage() {
    // RUNTIME ARGS START
    let source_nft_contract_address: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_NFT_CONTRACT_ADDRESS,
        BridgeError::MissingArgumentSourceNftContractAddress,
        BridgeError::InvalidArgumentSourceNftContractAddress,
    )
    .unwrap_or_revert();

    let storage_address: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_STORAGE_ADDRESS,
        BridgeError::MissingArgumentStorageAddress,
        BridgeError::InvalidArgumentStorageAddress,
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

    // GETTING SELF CHAIN START
    let self_chain_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );
    let self_chain: String = storage::read_or_revert(self_chain_ref);
    let self_chain_vec = self_chain.to_bytes().unwrap();
    // GETTING SELF CHAIN START

    let duplicate_to_original_dict_ref = utils::get_uref(
        KEY_DUPLICATE_TO_ORIGINAL_DICT,
        BridgeError::MissingDTODictRef,
        BridgeError::InvalidOTDDictRef,
    );

    let waiting_dict_ref = utils::get_uref(
        KEY_WAITING_DICT,
        BridgeError::MissingWaitingDictRef,
        BridgeError::InvalidWaitingDictRef,
    );

    let source_nft_contract_address_vec = source_nft_contract_address.to_bytes().unwrap();

    let mut mut_source_nft_contract_address_vec = source_nft_contract_address_vec.clone();
    mut_source_nft_contract_address_vec.extend(self_chain_vec.clone());
    let key = create_hash_key(mut_source_nft_contract_address_vec);

    let original_collection_address_option = storage::dictionary_get::<
        DuplicateToOriginalContractInfo,
    >(duplicate_to_original_dict_ref, key.as_str())
    .unwrap_or(None);

    match original_collection_address_option {
        Some(_) => {
            // notOriginal
            let duplicate_storage_dict_ref = utils::get_uref(
                KEY_DUPLICATE_STORAGE_DICT,
                BridgeError::MissingDSRef,
                BridgeError::InvalidDSRef,
            );
            let is_storage_exists_option =
                storage::dictionary_get::<ContractHash>(duplicate_storage_dict_ref, key.as_str())
                    .unwrap_or(None);

            match is_storage_exists_option {
                Some(_) => {
                    runtime::revert(BridgeError::DuplicateStorageAlreadyExists);
                }
                None => {
                    // SAVE STORAGE DATA
                    storage::dictionary_put(
                        duplicate_storage_dict_ref,
                        key.as_str(),
                        storage_address,
                    );

                    storage::dictionary_put(
                        waiting_dict_ref,
                        key.as_str(),
                        Waiting {
                            wait: false,
                            data_type: "storage".to_string(),
                        },
                    );
                }
            }
        }
        None => {
            // isOriginal
            let original_storage_dict_ref = utils::get_uref(
                KEY_ORIGINAL_STORAGE_DICT,
                BridgeError::MissingOSRef,
                BridgeError::InvalidOSRef,
            );
            let is_storage_exists_option =
                storage::dictionary_get::<ContractHash>(original_storage_dict_ref, key.as_str())
                    .unwrap_or(None);
            match is_storage_exists_option {
                Some(_) => {
                    runtime::revert(BridgeError::OriginalStorageAlreadyExists);
                }
                None => {
                    // SAVE STORAGE DATA
                    storage::dictionary_put(
                        original_storage_dict_ref,
                        key.as_str(),
                        storage_address,
                    );

                    storage::dictionary_put(
                        waiting_dict_ref,
                        key.as_str(),
                        Waiting {
                            wait: false,
                            data_type: "storage".to_string(),
                        },
                    );
                }
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

    let data = ClaimData {
        token_id: token_id.clone(),
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
    matches_current_chain(data.destination_chain.clone(), self_chain.clone());

    let nft_type_ref = utils::get_uref(
        KEY_TYPE_ERC721,
        BridgeError::MissingTypeErc721Ref,
        BridgeError::InvalidTypeErc721Ref,
    );
    let self_nft_type: String = storage::read_or_revert(nft_type_ref);
    if data.nft_type != self_nft_type {
        runtime::revert(BridgeError::InvalidNftType);
    }

    let claim_data_vec = data
        .to_bytes()
        .unwrap_or_revert_with(BridgeError::ClaimDataConversionError);
    let ss_key = create_hash_key(claim_data_vec);

    let mut submitted_sigs_data = get_done_info(ss_key.clone());
    let done_info = submitted_sigs_data.clone().1;

    let validators_to_rewards = submitted_sigs_data
        .clone()
        .0
        .into_iter()
        .map(|x| x.public_key)
        .collect::<Vec<_>>();

    if done_info.done && done_info.data_type == DataType::Claim {
        runtime::revert(BridgeError::DataAlreadyProcessed);
    }
    if !done_info.done && done_info.can_do {
        if !done_info.fee_taken {
            transfer_tx_fees(data.fee, sender_purse, Receiver::Bridge);
            reward_validators(data.fee, validators_to_rewards);
        }
    }
    if !done_info.can_do {
        runtime::revert(BridgeError::MoreSignaturesNeeded);
    }

    let original_to_duplicate_dict_ref = utils::get_uref(
        KEY_ORIGINAL_TO_DUPLICATE_DICT,
        BridgeError::MissingOTDDictRef,
        BridgeError::InvalidOTDDictRef,
    );

    let self_chain_vec = self_chain.to_bytes().unwrap();
    let source_chain_vec = data.source_chain.to_bytes().unwrap();

    let mut source_nft_contract_address_vec = vec![];

    if data.source_chain == self_chain {
        let contract =
            ContractHash::from_formatted_str(&*data.source_nft_contract_address).unwrap();
        source_nft_contract_address_vec = contract.to_bytes().unwrap();
    } else {
        source_nft_contract_address_vec = data.source_nft_contract_address.to_bytes().unwrap();
    }

    let mut mut_source_nft_contract_address_vec = source_nft_contract_address_vec.clone();
    mut_source_nft_contract_address_vec.extend(source_chain_vec.clone());
    let otd_key = create_hash_key(mut_source_nft_contract_address_vec);

    let waiting_dict_ref = utils::get_uref(
        KEY_WAITING_DICT,
        BridgeError::MissingWaitingDictRef,
        BridgeError::InvalidWaitingDictRef,
    );

    let waiting =
        storage::dictionary_get::<Waiting>(waiting_dict_ref, otd_key.as_str()).unwrap_or(None);

    match waiting {
        Some(v) => {
            if v.wait && v.data_type == "collection" {
                runtime::revert(BridgeError::WaitingForCollection);
            }
        }
        None => {}
    }

    let duplicate_collection_address_option = storage::dictionary_get::<
        OriginalToDuplicateContractInfo,
    >(
        original_to_duplicate_dict_ref, otd_key.as_str()
    )
    .unwrap_or(None);

    let mut duplicate_collection_address = OriginalToDuplicateContractInfo {
        chain: "".to_string(),
        contract_address: Default::default(),
    };

    // TOKEN
    let token_info_key_self_dict_ref = utils::get_uref(
        KEY_TOKEN_INFO_KEY_SELF_DICT,
        BridgeError::MissingTokenInfoKeySelfDictRef,
        BridgeError::InvalidTokenInfoKeySelfDictRef,
    );

    let token_info_get_self_dict_ref = utils::get_uref(
        KEY_TOKEN_INFO_GET_SELF_DICT,
        BridgeError::MissingTokenInfoGetSelfDictRef,
        BridgeError::InvalidTokenInfoGetSelfDictRef,
    );

    // 0 casper 0a234 ->  100 bsc 0x123
    let mut mut_token_id_vec = data.token_id.to_bytes().unwrap();
    mut_token_id_vec.extend(source_chain_vec);
    mut_token_id_vec.extend(source_nft_contract_address_vec.clone());

    let token_key = create_hash_key(mut_token_id_vec);

    let is_token_exists =
        storage::dictionary_get::<SelfToken>(token_info_get_self_dict_ref, token_key.as_str())
            .unwrap_or(None);
    // TOKEN

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

            let mut duplicate_collection_address_vec = duplicate_collection_address
                .contract_address
                .to_bytes()
                .unwrap();
            duplicate_collection_address_vec.extend(self_chain_vec.clone());
            let ds_key = create_hash_key(duplicate_collection_address_vec);

            storage_contract_option =
                storage::dictionary_get::<ContractHash>(duplicate_storage_ref, ds_key.as_str())
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
                storage::dictionary_get::<ContractHash>(original_storage_ref, otd_key.as_str())
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
        transfer_tx_fees(amount - data.fee, sender_purse, Receiver::Service);
    } else {
        if done_info.fee_taken {
            transfer_tx_fees(amount, sender_purse, Receiver::Sender);
        } else {
            transfer_tx_fees(amount - data.fee, sender_purse, Receiver::Sender);
        }
    }

    // ===============================/ hasDuplicate && hasStorage /=======================
    if has_duplicate && has_storage {
        match is_token_exists {
            Some(v) => {
                let nft_owner = owner_of(
                    duplicate_collection_address.contract_address,
                    v.token_id.clone(),
                );

                if nft_owner == Key::from(storage_contract) {
                    // UNLOCK NFT
                    unlock_token(
                        storage_contract,
                        v.token_id.clone(),
                        data.destination_user_address,
                    );
                    submitted_sigs_data.1.done = true;
                    submitted_sigs_data.1.fee_taken = true;
                    save_done_info(submitted_sigs_data, ss_key);

                    casper_event_standard::emit(Claimed::new(
                        data.lock_tx_chain,
                        data.source_chain,
                        data.transaction_hash,
                        duplicate_collection_address.contract_address.to_string(),
                        v.token_id,
                    ));
                }
            }
            None => {
                // MINT NFT
                register(
                    duplicate_collection_address.contract_address,
                    Key::from(data.destination_user_address),
                );
                let minted_token_id = mint(
                    duplicate_collection_address.contract_address,
                    Key::from(data.destination_user_address),
                    data.metadata,
                );
                submitted_sigs_data.1.done = true;
                submitted_sigs_data.1.fee_taken = true;
                save_done_info(submitted_sigs_data, ss_key);

                let mut mut_self_token_id = minted_token_id.to_bytes().unwrap();
                mut_self_token_id.extend(self_chain_vec);
                mut_self_token_id.extend(
                    duplicate_collection_address
                        .contract_address
                        .to_bytes()
                        .unwrap(),
                );
                let self_token_key = create_hash_key(mut_self_token_id);

                storage::dictionary_put(
                    token_info_key_self_dict_ref,
                    self_token_key.as_str(),
                    OtherToken {
                        token_id: data.token_id.clone(),
                        chain: data.source_chain.clone(),
                        contract_address: data.source_nft_contract_address,
                    },
                );

                storage::dictionary_put(
                    token_info_get_self_dict_ref,
                    token_key.as_str(),
                    SelfToken {
                        token_id: TokenIdentifier::from_bytes(
                            &*minted_token_id.to_bytes().unwrap(),
                        )
                        .unwrap()
                        .0,
                        chain: self_chain,
                        contract_address: duplicate_collection_address.contract_address,
                    },
                );

                casper_event_standard::emit(Claimed::new(
                    data.lock_tx_chain,
                    data.source_chain,
                    data.transaction_hash,
                    duplicate_collection_address.contract_address.to_string(),
                    TokenIdentifier::from_bytes(&*minted_token_id.to_bytes().unwrap())
                        .unwrap()
                        .0,
                ));
            }
        }
    }
    // ===============================/ hasDuplicate && NOT hasStorage /=======================
    else if has_duplicate && !has_storage {
        // MINT NFT
        register(
            duplicate_collection_address.contract_address,
            Key::from(data.destination_user_address),
        );
        let minted_token_id = mint(
            duplicate_collection_address.contract_address,
            Key::from(data.destination_user_address),
            data.metadata,
        );
        submitted_sigs_data.1.done = true;
        submitted_sigs_data.1.fee_taken = true;
        save_done_info(submitted_sigs_data, ss_key);

        let mut mut_self_token_id = minted_token_id.to_bytes().unwrap();
        mut_self_token_id.extend(self_chain_vec);
        mut_self_token_id.extend(
            duplicate_collection_address
                .contract_address
                .to_bytes()
                .unwrap(),
        );
        let self_token_key = create_hash_key(mut_self_token_id);
        storage::dictionary_put(
            token_info_key_self_dict_ref,
            self_token_key.as_str(),
            OtherToken {
                token_id: data.token_id.clone(),
                chain: data.source_chain.clone(),
                contract_address: data.source_nft_contract_address,
            },
        );

        storage::dictionary_put(
            token_info_get_self_dict_ref,
            token_key.as_str(),
            SelfToken {
                token_id: TokenIdentifier::from_bytes(&*minted_token_id.to_bytes().unwrap())
                    .unwrap()
                    .0,
                chain: self_chain,
                contract_address: duplicate_collection_address.contract_address,
            },
        );

        casper_event_standard::emit(Claimed::new(
            data.lock_tx_chain,
            data.source_chain,
            data.transaction_hash,
            duplicate_collection_address.contract_address.to_string(),
            TokenIdentifier::from_bytes(&*minted_token_id.to_bytes().unwrap())
                .unwrap()
                .0,
        ));
    }
    // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
    else if !has_duplicate && !has_storage {
        submitted_sigs_data.1.fee_taken = true;
        save_done_info(submitted_sigs_data, ss_key);

        storage::dictionary_put(
            waiting_dict_ref,
            otd_key.as_str(),
            Waiting {
                wait: true,
                data_type: "collection".to_string(),
            },
        );

        casper_event_standard::emit(DeployCollection::new(
            data.source_chain,
            data.source_nft_contract_address,
            data.name,
            data.symbol,
            data.royalty,
            data.royalty_receiver,
        ));
    }
    // ===============================/ NOT hasDuplicate && hasStorage /=======================
    else if !has_duplicate && has_storage {
        match is_token_exists {
            Some(v) => {
                let contract =
                    ContractHash::from_formatted_str(&*data.source_nft_contract_address).unwrap();

                let nft_owner = owner_of(contract, v.token_id.clone());

                if nft_owner == Key::from(storage_contract) {
                    // UNLOCK NFT
                    unlock_token(
                        storage_contract,
                        v.token_id.clone(),
                        data.destination_user_address,
                    );
                    submitted_sigs_data.1.done = true;
                    submitted_sigs_data.1.fee_taken = true;
                    save_done_info(submitted_sigs_data, ss_key);
                    casper_event_standard::emit(Claimed::new(
                        data.lock_tx_chain,
                        data.source_chain,
                        data.transaction_hash,
                        data.source_nft_contract_address,
                        v.token_id,
                    ));
                }
            }
            None => {
                // CANT BE THERE
                runtime::revert(BridgeError::CantBeThere);
            }
        }
    } else {
        runtime::revert(BridgeError::InvalidBridgeState);
    };
}
#[no_mangle]
pub extern "C" fn update_collection() {
    // RUNTIME ARGS START
    let source_chain: String = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_CHAIN,
        BridgeError::MissingArgumentSourceChain,
        BridgeError::InvalidArgumentSourceChain,
    )
    .unwrap_or_revert();

    let source_nft_contract_address: String = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_NFT_CONTRACT_ADDRESS,
        BridgeError::MissingArgumentSourceNftContractAddress,
        BridgeError::InvalidArgumentSourceNftContractAddress,
    )
    .unwrap_or_revert();

    let collection_address: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_ADDRESS,
        BridgeError::MissingArgumentCollectionAddress,
        BridgeError::InvalidArgumentCollectionAddress,
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

    let waiting_dict_ref = utils::get_uref(
        KEY_WAITING_DICT,
        BridgeError::MissingWaitingDictRef,
        BridgeError::InvalidWaitingDictRef,
    );

    let self_chain_vec = self_chain.to_bytes().unwrap();
    let source_chain_vec = source_chain.to_bytes().unwrap();

    let mut source_nft_contract_address_vec = vec![];

    if source_chain == self_chain {
        let contract = ContractHash::from_formatted_str(&*source_nft_contract_address).unwrap();
        source_nft_contract_address_vec = contract.to_bytes().unwrap();
    } else {
        source_nft_contract_address_vec = source_nft_contract_address.to_bytes().unwrap();
    }

    let mut mut_source_nft_contract_address_vec = source_nft_contract_address_vec.clone();
    mut_source_nft_contract_address_vec.extend(source_chain_vec.clone());
    let otd_key = create_hash_key(mut_source_nft_contract_address_vec);

    let is_collection_exists = storage::dictionary_get::<OriginalToDuplicateContractInfo>(
        original_to_duplicate_dict_ref,
        otd_key.as_str(),
    )
    .unwrap_or(None);

    match is_collection_exists {
        Some(_) => {
            runtime::revert(BridgeError::CollectionAlreadyExists);
        }
        None => {
            storage::dictionary_put(
                original_to_duplicate_dict_ref,
                otd_key.as_str(),
                OriginalToDuplicateContractInfo {
                    chain: self_chain.clone(),
                    contract_address: collection_address,
                },
            );

            let mut mut_collection_address_vec = collection_address.to_bytes().unwrap();
            mut_collection_address_vec.extend(self_chain_vec.clone());
            let dto_key = create_hash_key(mut_collection_address_vec);

            storage::dictionary_put(
                duplicate_to_original_dict_ref,
                dto_key.as_str(),
                DuplicateToOriginalContractInfo {
                    chain: source_chain,
                    contract_address: source_nft_contract_address,
                },
            );

            storage::dictionary_put(
                waiting_dict_ref,
                otd_key.as_str(),
                Waiting {
                    wait: false,
                    data_type: "collection".to_string(),
                },
            );
        }
    }
}
#[no_mangle]
pub extern "C" fn submit_signatures() {
    // RUNTIME ARGS START
    let data_hash: [u8; 32] = utils::get_named_arg_with_user_errors(
        ARG_DATA_HASH,
        BridgeError::MissingArgumentDataHash,
        BridgeError::InvalidArgumentDataHash,
    )
    .unwrap_or_revert();

    let data_type: DataType = utils::get_named_arg_with_user_errors(
        ARG_DATA_TYPE,
        BridgeError::MissingArgumentDataType,
        BridgeError::InvalidArgumentDataType,
    )
    .unwrap_or_revert();

    let signatures: Vec<Sigs> = utils::get_named_arg_with_user_errors(
        ARG_SIGNATURES,
        BridgeError::MissingArgumentSignatures,
        BridgeError::InvalidArgumentSignatures,
    )
    .unwrap_or_revert();
    // RUNTIME ARGS END

    submit_sigs(data_hash.to_bytes().unwrap(), data_type, signatures);
}
fn generate_entry_points() -> EntryPoints {
    let mut entrypoints = EntryPoints::new();

    let init = EntryPoint::new(
        ENTRY_POINT_BRIDGE_INITIALIZE,
        vec![
            Parameter::new(ARG_VALIDATOR, CLType::PublicKey),
            Parameter::new(ARG_CHAIN_TYPE, CLType::String),
            Parameter::new(ARG_SERVICE_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_SELF_HASH, CLType::ByteArray(32)),
            Parameter::new(ARG_STORAGE_DEPLOY_FEE, CLType::U512),
            Parameter::new(ARG_COLLECTION_DEPLOY_FEE, CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let add_validator = EntryPoint::new(
        ENTRY_POINT_BRIDGE_ADD_VALIDATOR,
        vec![Parameter::new(
            ARG_NEW_VALIDATOR_PUBLIC_KEY,
            CLType::ByteArray(32),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let blacklist_validator = EntryPoint::new(
        ENTRY_POINT_BRIDGE_BLACKLIST_VALIDATOR,
        vec![Parameter::new(
            ARG_VALIDATOR_PUBLIC_KEY,
            CLType::ByteArray(32),
        )],
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

    let update_storage = EntryPoint::new(
        ENTRY_POINT_UPDATE_STORAGE_AND_PROCESS_LOCK,
        vec![
            Parameter::new(ARG_SOURCE_NFT_CONTRACT_ADDRESS, CLType::ByteArray(32)),
            Parameter::new(ARG_STORAGE_ADDRESS, CLType::ByteArray(32)),
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
            Parameter::new(ARG_SENDER_PURSE, CLType::URef),
            Parameter::new(ARG_AMOUNT, CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let update_collection = EntryPoint::new(
        ENTRY_POINT_UPDATE_COLLECTION_AND_PROCESS_LOCK,
        vec![
            Parameter::new(ARG_SOURCE_CHAIN, CLType::String),
            Parameter::new(ARG_SOURCE_NFT_CONTRACT_ADDRESS, CLType::String),
            Parameter::new(ARG_COLLECTION_ADDRESS, CLType::ByteArray(32)),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let submit_signatures = EntryPoint::new(
        ENTRY_POINT_SUBMIT_SIGNATURES,
        vec![
            Parameter::new(ARG_DATA_HASH, CLType::ByteArray(32)),
            Parameter::new(ARG_DATA_TYPE, CLType::U8),
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

    let change_collection_deploy_fee = EntryPoint::new(
        ENTRY_POINT_CHANGE_COLLECTION_DEPLOY_FEE,
        vec![Parameter::new(ARG_COLLECTION_DEPLOY_FEE, CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let change_storage_deploy_fee = EntryPoint::new(
        ENTRY_POINT_CHANGE_STORAGE_DEPLOY_FEE,
        vec![Parameter::new(ARG_STORAGE_DEPLOY_FEE, CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entrypoints.add_entry_point(init);
    entrypoints.add_entry_point(add_validator);
    entrypoints.add_entry_point(blacklist_validator);
    entrypoints.add_entry_point(claim_reward_validator);
    entrypoints.add_entry_point(lock);
    entrypoints.add_entry_point(update_storage);
    entrypoints.add_entry_point(claim);
    entrypoints.add_entry_point(update_collection);
    entrypoints.add_entry_point(submit_signatures);
    entrypoints.add_entry_point(change_collection_deploy_fee);
    entrypoints.add_entry_point(change_storage_deploy_fee);
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
