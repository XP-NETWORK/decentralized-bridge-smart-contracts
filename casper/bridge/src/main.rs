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
    boxed::Box,
    string::{String, ToString},
    vec,
    vec::Vec,
};
// Importing aspects of the Casper platform.
use casper_contract::{
    contract_api::{
        self, account,
        runtime::{self},
        storage,
        system::{transfer_from_purse_to_account, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
// Importing specific Casper types.
use casper_types::{
    account::AccountHash, bytesrepr::{serialize, Bytes, FromBytes, ToBytes}, contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys}, system::{auction::ARG_AMOUNT, mint::ARG_TO}, CLType, ContractHash, Key, Parameter, PublicKey as CPublicKey, URef, U256, U512
};

use ed25519_dalek::{PublicKey, Signature, Verifier};
use entrypoints::*;
use errors::BridgeError;
use events::{TransferNftEvent, UnfreezeNftEvent};
use external::xp_nft::{burn, mint, transfer, TokenIdentifier};
use keys::*;
use sha2::{Digest, Sha512};
use structs::{
    AddValidator, FreezeNFT, PauseData, SignerAndSignature, TxFee, UnpauseData, UpdateGroupKey, ValidateBlacklist, ValidateTransferData, ValidateUnfreezeData, ValidateWhitelist, Validator, WithdrawFeeData, WithdrawNFT
};

pub const INITIALIZED: &str = "initialized";
pub const THIS_CONTRACT: &str = "this_contract";
pub const INSTALLER: &str = "installer";

// RUNTIME ARGS
pub const ARG_VALIDATORS: &str = "validators_arg";
pub const ARG_CHAIN_TYPE: &str = "chain_type_arg";
pub const ARG_COLLECTION_DEPLOYER: &str = "collection_deployer_arg";
pub const ARG_STORAGE_DEPLOYER: &str = "storage_deployer_arg";
pub const ARG_NEW_VALIDATOR: &str = "add_validator_args";

pub const KEY_PURSE: &str = "bridge_purse";
pub const HASH_KEY_NAME: &str = "bridge_package";
pub const ACCESS_KEY_NAME: &str = "access_key_name_bridge";
pub const KEY_CHAIN_TYPE: &str = "chain_type";
pub const KEY_COLLECTION_DEPLOYER: &str = "collection_deployer";
pub const KEY_STORAGE_DEPLOYER: &str = "storage_deployer";
pub const KEY_VALIDATORS_COUNT: &str = "validators_count";

/// Ed25519 Signature verification logic.
/// Signature check for bridge actions.
/// Consumes the passed action_id.
// fn require_sig(action_id: U256, data: Vec<u8>, sig_data: &[u8], context: &[u8]) {
//     let f = check_consumed_action(&action_id);

//     if !f {
//         runtime::revert(BridgeError::RetryingConsumedActions);
//     }

//     insert_consumed_action(&action_id);

//     let mut hasher = Sha512::new();
//     hasher.update(context);
//     hasher.update(data);
//     let hash = hasher.finalize();

//     let group_key = get_group_key();

//     let sig = Signature::new(
//         sig_data
//             .try_into()
//             .map_err(|_| BridgeError::FailedToPrepareSignature)
//             .unwrap_or_revert(),
//     );
//     let key = PublicKey::from_bytes(group_key.as_slice())
//         .map_err(|_| BridgeError::FailedToPreparePublicKey)
//         .unwrap_or_revert();
//     let res = key.verify(&hash, &sig);
//     if res.is_err() {
//         runtime::revert(BridgeError::UnauthorizedAction);
//     }
// }

/// Ed25519 Signature verification logic.
fn verify_signature(data: Vec<u8>, signature: &[u8], key: &[u8]) -> bool {

    let mut hasher = Sha512::new();

    hasher.update(data);

    let hash = hasher.finalize();

    let sig = Signature::new(
        signature
                .try_into()
                .map_err(|_| BridgeError::FailedToPrepareSignature)
                .unwrap_or_revert(),
            );
    
    let key = PublicKey::from_bytes(key)
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

    let validator: Bytes = runtime::get_named_arg(ARG_VALIDATORS);
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

    let validator_public_key = CPublicKey::from_bytes(&validator).unwrap().0.to_string();

    storage::dictionary_put(validators_dict, &validator_public_key, Validator {
        added: true,
        pending_rewards: U256::from(0)
    });
    runtime::put_key(KEY_VALIDATORS_COUNT, storage::new_uref(1).into());

}

#[no_mangle]
pub extern "C" fn add_validator() {
    
    let data: AddValidator = utils::get_named_arg_with_user_errors(
        ARG_NEW_VALIDATOR,
        BridgeError::MissingArgumentContract,
        BridgeError::InvalidArgumentContract,
    )
    .unwrap_or_revert();

    let validators_dict = storage::new_dictionary(KEY_VALIDATORS_DICT)
        .unwrap_or_revert_with(BridgeError::FailedToCreateDictionary);

    let mut uv: Vec<Bytes>;

    let mut percentage: u64 = 0;

    for arg in data.signatures.into_iter() {

        let valid = verify_signature(
            data.new_validator_public_key.to_bytes().unwrap(), 
            arg.signature.to_bytes().unwrap().as_slice(),
            arg.public_key.to_bytes().unwrap().as_slice(),
        );
        if valid {
            let validator_exists = storage::dictionary_get(validators_dict, AccountHash::from_public_key(public_key, blake2b_hash_fn));

        }

    }

    let paused_uref = utils::get_uref(
        KEY_PAUSED,
        BridgeError::MissingGroupKeyUref,
        BridgeError::InvalidGroupKeyUref,
    );

    storage::write(paused_uref, true)
}


#[no_mangle]
pub extern "C" fn validate_pause() {
    let action_id: U256 = utils::get_named_arg_with_user_errors(
        ARG_ACTION_ID,
        BridgeError::MissingArgumentActionID,
        BridgeError::InvalidArgumentActionID,
    )
    .unwrap_or_revert();
    let data = PauseData { action_id };

    let sig_data: [u8; 64] = utils::get_named_arg_with_user_errors(
        ARG_SIG_DATA,
        BridgeError::MissingArgumentSigData,
        BridgeError::InvalidArgumentSigData,
    )
    .unwrap_or_revert();

    require_sig(
        data.action_id,
        data.to_bytes()
            .unwrap_or_revert_with(BridgeError::FailedToSerializeActionStruct),
        &sig_data,
        b"SetPause",
    );

    let paused_uref = utils::get_uref(
        KEY_PAUSED,
        BridgeError::MissingGroupKeyUref,
        BridgeError::InvalidGroupKeyUref,
    );

    storage::write(paused_uref, true)
}


pub fn transfer_tx_fees(amount: U512) {
    let this_uref = utils::get_uref(
        KEY_PURSE,
        BridgeError::MissingThisContractUref,
        BridgeError::InvalidThisContractUref,
    );

    let this_contract: URef = storage::read_or_revert(this_uref);

    transfer_from_purse_to_purse(account::get_main_purse(), this_contract, amount, None)
        .unwrap_or_revert();
}

fn generate_entry_points() -> EntryPoints {
    let mut entrypoints = EntryPoints::new();

    let init = EntryPoint::new(
        ENTRY_POINT_BRIDGE_INITIALIZE,
        vec![
            Parameter::new(ARG_GROUP_KEY, CLType::ByteArray(32)),
            Parameter::new(ARG_FEE_PUBLIC_KEY, CLType::ByteArray(32)),
            Parameter::new(ARG_WHITELIST, CLType::List(Box::new(CLType::ByteArray(32)))),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let validate_pause = EntryPoint::new(
        ENTRY_POINT_BRIDGE_VALIDATE_PAUSE,
        vec![
            Parameter::new(ARG_ACTION_ID, CLType::U256),
            Parameter::new(ARG_SIG_DATA, CLType::ByteArray(64)),
        ],
        CLType::Bool,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    

    entrypoints.add_entry_point(init); // Not needed
    entrypoints.add_entry_point(validate_pause); // Done
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
