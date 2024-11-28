#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// This code imports necessary aspects of external crates that we will use in our contract code.
extern crate alloc;

mod endpoints;
mod errors;
mod keys;
mod utils;

// Importing Rust types.
use alloc::string::String;
use alloc::{string::ToString, vec};
// Importing aspects of the Casper platform.
use casper_contract::{
    contract_api::{
        runtime::{self},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
// Importing specific Casper types.
use casper_types::account::AccountHash;
use casper_types::bytesrepr::ToBytes;
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    CLType, ContractHash, Key, Parameter,
};
use common::collection::{owner_of, transfer, TokenIdentifier};
use endpoints::*;
use errors::StorageError;
use keys::*;

#[no_mangle]
pub extern "C" fn init() {
    if utils::named_uref_exists(INITIALIZED) {
        runtime::revert(StorageError::AlreadyInitialized);
    }

    let collection: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION,
        StorageError::MissingArgumentCollection,
        StorageError::InvalidArgumentCollection,
    )
    .unwrap_or_revert();

    let owner: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_OWNER,
        StorageError::MissingArgumentOwner,
        StorageError::InvalidArgumentOwner,
    )
    .unwrap_or_revert();

    let self_hash: ContractHash = utils::get_named_arg_with_user_errors(
        ARG_SELF_HASH,
        StorageError::MissingArgumentSelfHash,
        StorageError::InvalidArgumentSelfHash,
    )
    .unwrap_or_revert();

    runtime::put_key(INITIALIZED, storage::new_uref(true).into());
    runtime::put_key(KEY_COLLECTION, storage::new_uref(collection).into());
    runtime::put_key(KEY_OWNER, storage::new_uref(owner).into());
    runtime::put_key(KEY_SELF_HASH, storage::new_uref(self_hash).into());
}

#[no_mangle]
pub extern "C" fn unlock_token() {
    let token_id: TokenIdentifier = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        StorageError::MissingArgumentTokenId,
        StorageError::InvalidArgumentTokenId,
    )
    .unwrap_or_revert();

    let to: AccountHash = utils::get_named_arg_with_user_errors(
        ARG_TO,
        StorageError::MissingArgumentTo,
        StorageError::InvalidArgumentTo,
    )
    .unwrap_or_revert();

    let collection_ref = utils::get_uref(
        KEY_COLLECTION,
        StorageError::MissingCollectionRef,
        StorageError::InvalidCollectionRef,
    );

    let self_hash_ref = utils::get_uref(
        KEY_SELF_HASH,
        StorageError::MissingSelfHashRef,
        StorageError::InvalidSelfHashRef,
    );

    let owner_ref = utils::get_uref(
        KEY_OWNER,
        StorageError::MissingOwnerRef,
        StorageError::InvalidOwnerRef,
    );

    let owner: ContractHash = storage::read_or_revert(owner_ref);

    let caller_contract = runtime::get_call_stack()
        .iter()
        .nth_back(1)
        .unwrap_or_revert_with(StorageError::FailedToGetCallStack)
        .contract_hash()
        .unwrap_or_revert_with(StorageError::FailedToParseContractHash)
        .clone();

    if Key::from(owner) != Key::from(caller_contract) {
        runtime::revert(StorageError::OnlyOwnerCanCallThisFunction);
    }

    let self_hash: ContractHash = storage::read_or_revert(self_hash_ref);

    let collection: ContractHash = storage::read_or_revert(collection_ref);

    let nft_owner = owner_of(collection, token_id.clone());

    let this_contract_key = Key::from(self_hash);

    let to_key = Key::from(to);

    if nft_owner == this_contract_key {
        transfer(collection, this_contract_key, to_key, token_id);
    } else {
        runtime::revert(StorageError::ThisContractIsNotTheOwnerOfThisToken);
    }
}

fn generate_entry_points() -> EntryPoints {
    let mut entrypoints = EntryPoints::new();

    let init = EntryPoint::new(
        ENTRY_POINT_STORAGE_INITIALIZE,
        vec![
            Parameter::new(ARG_COLLECTION, CLType::ByteArray(32)),
            Parameter::new(ARG_OWNER, CLType::ByteArray(32)),
            Parameter::new(ARG_SELF_HASH, CLType::ByteArray(32)),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let unlock_token = EntryPoint::new(
        ENTRY_POINT_STORAGE_UNLOCK_TOKEN,
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_TO, CLType::ByteArray(32)),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entrypoints.add_entry_point(init);
    entrypoints.add_entry_point(unlock_token);
    entrypoints
}

fn install_storage() {
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

    let s: String = runtime::get_named_arg("source_nft_contract_address");

    runtime::put_key(&(THIS_CONTRACT.to_string() + &s), contract_hash.into());
}

#[no_mangle]
pub extern "C" fn call() {
    install_storage();
}
