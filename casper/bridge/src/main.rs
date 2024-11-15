#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// This code imports necessary aspects of external crates that we will use in our contract code.
extern crate alloc;

mod entrypoints;
mod errors;
mod events;
mod external;
mod keys;
mod structs;
mod utils;

use core::convert::TryInto;

// Importing Rust types.
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
// Importing aspects of the Casper platform.
use casper_contract::{
    contract_api::{
        runtime::{self},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_contract::contract_api::system::{get_purse_balance, transfer_from_purse_to_purse, transfer_to_public_key};
// Importing specific Casper types.
use casper_types::{bytesrepr::ToBytes, contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys}, AsymmetricType, CLType, CLTyped, ContractHash, Key, Parameter, PublicKey as CPublicKey, URef, U256, U512};
use casper_types::bytesrepr::{Bytes, FromBytes};
use casper_types::contracts::FromStrError::AccountHash;
use ed25519_dalek::{PublicKey as EPublicKey, Signature, Verifier};
use entrypoints::*;
use errors::BridgeError;
use events::AddNewValidator;
use keys::*;
use sha2::{Digest, Sha512};
use structs::{
    AddValidator, Validator,
};
use crate::external::xp_nft::{transfer, TokenIdentifier};
use crate::structs::ContractInfo;

type Sigs = (CPublicKey, [u8; 64]);

fn matches_current_chain(destination_chain: String) {
    let self_chain_ref = utils::get_uref(
        KEY_CHAIN_TYPE,
        BridgeError::MissingChainTypeRef,
        BridgeError::InvalidChainTypeRef,
    );
    let self_chain: String = storage::read_or_revert(self_chain_ref);

    if destination_chain != self_chain {
        runtime::revert(BridgeError::InvalidDestinationChain);
    }
}
fn has_correct_fee(fee: U256, msg_value: U256) {
    if msg_value < fee {
        runtime::revert(BridgeError::FeeLessThanSentAmount);
    }
}
/// Ed25519 Signature verification logic.
fn verify_signatures(data: Vec<u8>, signature: &[u8], key: &[u8]) -> bool {
    let mut hasher = Sha512::new();

    hasher.update(data);

    let hash = hasher.finalize();

    let sig = Signature::new(
        signature
            .try_into()
            .map_err(|_| BridgeError::FailedToPrepareSignature)
            .unwrap_or_revert(),
    );

    let key = EPublicKey::from_bytes(key)
        .map_err(|_| BridgeError::FailedToPreparePublicKey)
        .unwrap_or_revert();

    let res = key.verify(&hash, &sig);

    if res.is_err() {
        return false;
    }
    true
}
fn check_storage(storage_dict_ref: URef, source_nft_contract_address: ContractHash, token_id: TokenIdentifier, key: &str) {
    let storage_address_option = storage::dictionary_get::<ContractHash>(storage_dict_ref, key).unwrap_or(None);

    match storage_address_option {
        Some(v) => {
            // TRANSFER TO STORAGE
            transfer_to_storage(v, source_nft_contract_address, token_id);
        }
        None => {
            // EMIT CREATE STORAGE EVENT
        }
    }
}

fn transfer_to_storage(storage_address: ContractHash, source_nft_contract_address: ContractHash, token_id: TokenIdentifier) {
    let this_ref = utils::get_uref(
        THIS_CONTRACT,
        BridgeError::MissingThisContractRef,
        BridgeError::InvalidThisContractRef,
    );

    let this_contract: ContractHash = storage::read_or_revert(this_ref);
    transfer(
        source_nft_contract_address,
        this_contract.into(),
        storage_address.into(),
        token_id,
    );
}
#[no_mangle]
pub extern "C" fn init() {
    if utils::named_uref_exists(INITIALIZED) {
        runtime::revert(BridgeError::AlreadyInitialized);
    }

    let validator: CPublicKey = runtime::get_named_arg(ARG_VALIDATORS);
    let chain_type: String = runtime::get_named_arg(ARG_CHAIN_TYPE);
    let collection_deployer: ContractHash = runtime::get_named_arg(ARG_COLLECTION_DEPLOYER);
    let storage_deployer: ContractHash = runtime::get_named_arg(ARG_STORAGE_DEPLOYER);
    let service_account: CPublicKey = runtime::get_named_arg(ARG_SERVICE_ACCOUNT);

    runtime::put_key(INITIALIZED, storage::new_uref(true).into());
    runtime::put_key(KEY_CHAIN_TYPE, storage::new_uref(chain_type).into());
    runtime::put_key(KEY_COLLECTION_DEPLOYER, storage::new_uref(collection_deployer).into());
    runtime::put_key(KEY_STORAGE_DEPLOYER, storage::new_uref(storage_deployer).into());
    runtime::put_key(KEY_SERVICE_ACCOUNT, storage::new_uref(service_account).into());


    // INITIALIZING BOOTSTRAP VALIDATOR
    let validators_dict = storage::new_dictionary(KEY_VALIDATORS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    let validator_public_key = &validator.to_string();

    storage::dictionary_put(validators_dict, &validator_public_key, Validator {
        added: true,
        pending_rewards: U256::from(0),
    });

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
    ).unwrap_or_revert();


    let signatures: Vec<Sigs> = utils::get_named_arg_with_user_errors(
        ARG_SIGNATURES,
        BridgeError::MissingArgumentSignatures,
        BridgeError::InvalidArgumentSignatures,
    ).unwrap_or_revert();

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingValidatorsDictRef,
        BridgeError::InvalidValidatorsDictRef,
    );
    let mut uv: Vec<CPublicKey> = [].to_vec();

    let mut percentage: u64 = 0;

    let data = new_validator_public_key.to_bytes().unwrap().iter().skip(1).cloned().collect::<Vec<u8>>();

    for arg in signatures.into_iter() {
        let key = arg.0.to_bytes().unwrap().iter().skip(1).cloned().collect::<Vec<u8>>();

        let sig = arg.1.to_bytes().unwrap();

        let valid = verify_signatures(
            data.clone(),
            sig.as_slice(),
            key.as_slice(),
        );
        if valid {
            let validator_exists = storage::dictionary_get::<Validator>(validators_dict_ref, &arg.0.to_string()).unwrap_or(None);

            match validator_exists {
                Some(v) => {
                    if v.added && !uv.contains(&arg.0) {
                        percentage = percentage + 1;
                        uv.push(
                            arg.0,
                        );
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
        storage::dictionary_put(validators_dict_ref, &new_validator_public_key.to_string(), Validator {
            added: true,
            pending_rewards: U256::from(0),
        });

        storage::write(validators_count_ref, validators_count + 1);

        casper_event_standard::emit(AddNewValidator::new(new_validator_public_key));
    } else {
        runtime::revert(BridgeError::ThresholdNotReached);
    }
}
#[no_mangle]
pub extern "C" fn lock() {
    let token_id: TokenIdentifier = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        BridgeError::MissingArgumentTokenId,
        BridgeError::InvalidArgumentTokenId,
    ).unwrap_or_revert();

    let destination_chain: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_CHAIN,
        BridgeError::MissingArgumentDestinationChain,
        BridgeError::InvalidArgumentDestinationChain,
    ).unwrap_or_revert();

    let destination_user_address: String = utils::get_named_arg_with_user_errors(
        ARG_DESTINATION_USER_ADDRESS,
        BridgeError::MissingArgumentDestinationUserAddress,
        BridgeError::InvalidArgumentDestinationUserAddress,
    ).unwrap_or_revert();

    let source_nft_contract_address: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_SOURCE_NFT_CONTRACT_ADDRESS,
        BridgeError::MissingArgumentSourceNftContractAddress,
        BridgeError::InvalidArgumentSourceNftContractAddress,
    ).unwrap_or_revert();

    let metadata_uri: String = utils::get_named_arg_with_user_errors(
        ARG_METADATA_URI,
        BridgeError::MissingArgumentMetadataUri,
        BridgeError::InvalidArgumentMetadataUri,
    ).unwrap_or_revert();

    let service_account_ref = utils::get_uref(
        ARG_SERVICE_ACCOUNT,
        BridgeError::MissingServiceAccountRef,
        BridgeError::MissingServiceAccountRef,
    );

    let service_account: CPublicKey = storage::read_or_revert(service_account_ref);

    let res = transfer_to_public_key(service_account, U512::from(10u32.pow(9)), 1.into());

    // let balance: U512 = get_purse_balance(caller_purse).unwrap_or_revert_with(BridgeError::CouldntGetPurseBalance);
    // let self_chain_ref = utils::get_uref(
    //     KEY_CHAIN_TYPE,
    //     BridgeError::MissingChainTypeRef,
    //     BridgeError::InvalidChainTypeRef,
    // );
    //
    // let self_chain: String = storage::read_or_revert(self_chain_ref);
    //
    // let duplicate_to_original_dict_ref = utils::get_uref(
    //     KEY_DUPLICATE_TO_ORIGINAL_DICT,
    //     BridgeError::MissingOTDDictRef,
    //     BridgeError::InvalidOTDDictRef,
    // );
    //
    // let key = &*(source_nft_contract_address.to_string() + self_chain.as_str());
    //
    // let original_collection_address_option = storage::dictionary_get::<ContractInfo>(duplicate_to_original_dict_ref, key).unwrap_or(None);
    //
    // match original_collection_address_option {
    //     Some(v) => {
    //         // notOriginal
    //         let duplicate_storage_dict_ref = utils::get_uref(
    //             KEY_DUPLICATE_STORAGE_DICT,
    //             BridgeError::MissingDSRef,
    //             BridgeError::InvalidDSRef,
    //         );
    //         check_storage(duplicate_storage_dict_ref, source_nft_contract_address, token_id, key);
    //     }
    //     None => {
    //         // isOriginal
    //     }
    // }
}


fn generate_entry_points() -> EntryPoints {
    let mut entrypoints = EntryPoints::new();

    let init = EntryPoint::new(
        ENTRY_POINT_BRIDGE_INITIALIZE,
        vec![
            Parameter::new(ARG_VALIDATORS, CLType::PublicKey),
            Parameter::new(ARG_CHAIN_TYPE, CLType::String),
            Parameter::new(ARG_COLLECTION_DEPLOYER, CLType::ByteArray(32)),
            Parameter::new(ARG_STORAGE_DEPLOYER, CLType::ByteArray(32)),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let add_validator = EntryPoint::new(
        ENTRY_POINT_BRIDGE_ADD_VALIDATOR,
        vec![
            Parameter::new(ARG_NEW_VALIDATOR_PUBLIC_KEY, CLType::ByteArray(32)),
            Parameter::new(ARG_SIGNATURES, CLType::List(Box::new(CLType::Tuple2([Box::new(CLType::PublicKey), Box::new(CLType::ByteArray(64))])))),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entrypoints.add_entry_point(init);
    entrypoints.add_entry_point(add_validator);
    entrypoints
}
fn install_contract() {
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
    install_contract();
}
