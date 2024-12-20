use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use casper_contract::{
    contract_api::{self, runtime},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_event_standard::Schemas;
use casper_types::{
    api_error,
    bytesrepr::{self, FromBytes, ToBytes},
    ApiError, ContractHash, Key, URef,
};

use crate::errors::BridgeError;
use crate::events::{
    AddNewValidator, BlackListValidator, Claimed, CollectionDeployFee, DeployCollection,
    DeployStorage, Locked, RewardValidator, StorageDeployFee,
};

// Initializes events-related named keys and records all event schemas.
pub fn init_events() {
    let schemas = Schemas::new()
        .with::<AddNewValidator>()
        .with::<DeployStorage>()
        .with::<Locked>()
        .with::<DeployCollection>()
        .with::<Claimed>()
        .with::<BlackListValidator>()
        .with::<RewardValidator>()
        .with::<CollectionDeployFee>()
        .with::<StorageDeployFee>();

    casper_event_standard::init(schemas);
}

pub(crate) fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

pub fn named_uref_exists(name: &str) -> bool {
    let (name_ptr, name_size, _bytes) = to_ptr(name);
    let mut key_bytes = vec![0u8; Key::max_serialized_length()];
    let mut total_bytes: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_key(
            name_ptr,
            name_size,
            key_bytes.as_mut_ptr(),
            key_bytes.len(),
            &mut total_bytes as *mut usize,
        )
    };

    api_error::result_from(ret).is_ok()
}

pub fn get_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    missing: BridgeError,
    invalid: BridgeError,
) -> Result<T, BridgeError> {
    let arg_size = get_named_arg_size(name).ok_or(missing)?;
    let arg_bytes = if arg_size > 0 {
        let res = {
            let data_non_null_ptr = contract_api::alloc_bytes(arg_size);
            let ret = unsafe {
                ext_ffi::casper_get_named_arg(
                    name.as_bytes().as_ptr(),
                    name.len(),
                    data_non_null_ptr.as_ptr(),
                    arg_size,
                )
            };
            let data =
                unsafe { Vec::from_raw_parts(data_non_null_ptr.as_ptr(), arg_size, arg_size) };
            api_error::result_from(ret).map(|_| data)
        };
        // Assumed to be safe as `get_named_arg_size` checks the argument already
        res.unwrap_or_revert_with(BridgeError::FailedToGetArgBytes)
    } else {
        // Avoids allocation with 0 bytes and a call to get_named_arg
        Vec::new()
    };

    bytesrepr::deserialize(arg_bytes).map_err(|_| invalid)
}

pub fn get_named_arg_size(name: &str) -> Option<usize> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => Some(arg_size),
        Err(ApiError::MissingArgument) => None,
        Err(e) => runtime::revert(e),
    }
}

pub(crate) fn get_key_with_user_errors(
    name: &str,
    missing: BridgeError,
    invalid: BridgeError,
) -> Key {
    let (name_ptr, name_size, _bytes) = to_ptr(name);
    let mut key_bytes = vec![0u8; Key::max_serialized_length()];
    let mut total_bytes: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_key(
            name_ptr,
            name_size,
            key_bytes.as_mut_ptr(),
            key_bytes.len(),
            &mut total_bytes as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => {}
        Err(ApiError::MissingKey) => runtime::revert(missing),
        Err(e) => runtime::revert(e),
    }
    key_bytes.truncate(total_bytes);

    bytesrepr::deserialize(key_bytes).unwrap_or_revert_with(invalid)
}

pub fn get_uref(name: &str, missing: BridgeError, invalid: BridgeError) -> URef {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_uref()
        .unwrap_or_revert_with(BridgeError::UnexpectedKeyVariant)
}
pub fn encode_dictionary_item_key(key: Key) -> String {
    match key {
        Key::Account(account_hash) => account_hash.to_string(),
        Key::Hash(hash_addr) => ContractHash::new(hash_addr).to_string(),
        _ => runtime::revert(BridgeError::InvalidKey),
    }
}
