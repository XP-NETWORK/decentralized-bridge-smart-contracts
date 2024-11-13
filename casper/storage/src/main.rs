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
// Importing specific Casper types.
use casper_types::{bytesrepr::ToBytes, contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys}, AsymmetricType, CLType, CLTyped, ContractHash, Parameter, PublicKey as CPublicKey, U256, U512};
use casper_types::bytesrepr::{Bytes, FromBytes};
use ed25519_dalek::{PublicKey as EPublicKey, Signature, Verifier};
use entrypoints::*;
use errors::BridgeError;
use events::AddNewValidator;
use keys::*;
use sha2::{Digest, Sha512};
use structs::{
    AddValidator, Validator
};

pub const INITIALIZED: &str = "initialized";
pub const THIS_CONTRACT: &str = "bridge_contract";
pub const INSTALLER: &str = "installer";

// RUNTIME ARGS INITIALIZE
pub const ARG_VALIDATORS: &str = "bootstrap_validator_arg";
pub const ARG_CHAIN_TYPE: &str = "chain_type_arg";
pub const ARG_COLLECTION_DEPLOYER: &str = "collection_deployer_arg";
pub const ARG_STORAGE_DEPLOYER: &str = "storage_deployer_arg";

// RUNTIME ARGS ADD VALIDATOR
pub const ARG_NEW_VALIDATOR_PUBLIC_KEY: &str = "new_validator_public_key_arg";
pub const ARG_SIGNATURES: &str = "signatures_arg";

pub const KEY_PURSE: &str = "bridge_purse";
pub const HASH_KEY_NAME: &str = "bridge_package";
pub const ACCESS_KEY_NAME: &str = "access_key_name_bridge";
pub const KEY_CHAIN_TYPE: &str = "chain_type";
pub const KEY_COLLECTION_DEPLOYER: &str = "collection_deployer";
pub const KEY_STORAGE_DEPLOYER: &str = "storage_deployer";
pub const KEY_VALIDATORS_COUNT: &str = "validators_count";

type Sigs = (CPublicKey, [u8; 64]);
/// Ed25519 Signature verification logic.
fn verify_signastures(data: Vec<u8>, signature: &[u8], key: &[u8]) -> bool {

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
        return false
    }
    true
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

    runtime::put_key(INITIALIZED, storage::new_uref(true).into());
    runtime::put_key(KEY_CHAIN_TYPE, storage::new_uref(chain_type).into());
    runtime::put_key(KEY_COLLECTION_DEPLOYER, storage::new_uref(collection_deployer).into());
    runtime::put_key(KEY_STORAGE_DEPLOYER, storage::new_uref(storage_deployer).into());

    let validators_dict = storage::new_dictionary(KEY_VALIDATORS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    storage::new_dictionary(KEY_BLACKLIST_VALIDATORS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    let validator_public_key = &validator.to_string();

    storage::dictionary_put(validators_dict, &validator_public_key, Validator {
        added: true,
        pending_rewards: U256::from(0)
    });
    runtime::put_key(KEY_VALIDATORS_COUNT, storage::new_uref(1u64).into());

}

#[no_mangle]
pub extern "C" fn add_validator() {

    let new_validator_public_key: CPublicKey = utils::get_named_arg_with_user_errors(
        ARG_NEW_VALIDATOR_PUBLIC_KEY,
        BridgeError::MissingArgumentContract,
        BridgeError::InvalidArgumentContract,
    ).unwrap_or_revert();


    let signatures: Vec<Sigs> = utils::get_named_arg_with_user_errors(
        ARG_SIGNATURES,
        BridgeError::MissingArgumentContract,
        BridgeError::InvalidArgumentContract,
    ).unwrap_or_revert();

    let validators_dict_ref = utils::get_uref(
        KEY_VALIDATORS_DICT,
        BridgeError::MissingConsumedActionsUref,
        BridgeError::InvalidConsumedActionsUref,
    );
    let mut uv: Vec<CPublicKey> = [].to_vec();

    let mut percentage: u64 = 0;

    let data = new_validator_public_key.to_bytes().unwrap().iter().skip(1).cloned().collect::<Vec<u8>>();

    for arg in signatures.into_iter() {

        let key = arg.0.to_bytes().unwrap().iter().skip(1).cloned().collect::<Vec<u8>>();

        let sig = arg.1.to_bytes().unwrap();

        let valid = verify_signastures(
            data.clone(),
            sig.as_slice(),
            key.as_slice(),
        );

        if valid {
            let validator_exists = storage::dictionary_get::<Validator>(validators_dict_ref, &arg.0.to_string())
                .map_err(|_| BridgeError::FailedToGetValidator)
                .unwrap_or_revert();

            match validator_exists {
                Some(v) => {
                    if v.added && !uv.contains(&arg.0) {
                        percentage = percentage + 1;
                        uv.push(
                            arg.0,
                        );
                    }
                },
                None => {

                },
            }
        }
    }

    let validators_count_ref = utils::get_uref(
        KEY_VALIDATORS_COUNT,
        BridgeError::MissingGroupKeyUref,
        BridgeError::InvalidGroupKeyUref,
    );
    let validators_count: u64 = storage::read_or_revert(validators_count_ref);

    if percentage >= (((validators_count * 2) / 3) + 1) {

        storage::dictionary_put(validators_dict_ref, &new_validator_public_key.to_string(), Validator {
            added: true,
            pending_rewards: U256::from(0)
        });

        storage::write(validators_count_ref, validators_count + 1);

        // let ev = AddNewValidator {
        //     public_key: new_validator_public_key.clone(),
        // };
        // casper_event_standard::emit(ev);
    }
    else {
        // "Threshold not reached!"
        runtime::revert(BridgeError::ThresholdNotReached);
    }
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
