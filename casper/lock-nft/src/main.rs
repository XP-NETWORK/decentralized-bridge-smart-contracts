#![no_std]
#![no_main]

mod errors;

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;

use casper_contract::{
    contract_api::{
        runtime,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_contract::contract_api::system::transfer_to_public_key;
use casper_types::{runtime_args, ContractHash, RuntimeArgs, U512};
use casper_types::account::AccountHash;
use casper_types::bytesrepr::{FromBytes};
use casper_types::PublicKey;
use crate::errors::LockError;
const ARG_BRIDGE_CONTRACT_HASH: &str = "bridge_contract";
pub const ARG_TOKEN_ID: &str = "token_id_arg";
pub const ARG_DESTINATION_CHAIN: &str = "destination_chain_arg";
pub const ARG_DESTINATION_USER_ADDRESS: &str = "destination_user_address_arg";
pub const ARG_SOURCE_NFT_CONTRACT_ADDRESS: &str = "source_nft_contract_address_arg";
pub const ARG_METADATA: &str = "metadata_arg";
pub const ARG_STORAGE_ADDRESS: &str = "storage_address_arg";
pub const ARG_NFT_SENDER_ADDRESS: &str = "nft_sender_address_arg";
const ARG_AMOUNT: &str = "amount";

#[no_mangle]
pub extern "C" fn call() {
    let bridge_contract_hash: ContractHash = runtime::get_named_arg::<ContractHash>(ARG_BRIDGE_CONTRACT_HASH);
    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);
    let destination_chain: String = runtime::get_named_arg(ARG_DESTINATION_CHAIN);
    let destination_user_address: String = runtime::get_named_arg(ARG_DESTINATION_USER_ADDRESS);
    let source_nft_contract_address: ContractHash = runtime::get_named_arg(ARG_SOURCE_NFT_CONTRACT_ADDRESS);
    let metadata: String = runtime::get_named_arg(ARG_METADATA);
    let storage_address: Option<ContractHash> = runtime::get_named_arg(ARG_STORAGE_ADDRESS);
    let nft_sender_address: Option<AccountHash> = runtime::get_named_arg(ARG_NFT_SENDER_ADDRESS);

    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    const ENTRY_POINT_LOCK_NFT: &str = "lock";

    let hex_service_account_string = "01e8a472f805d3c4f01644cbd75bc467a73d14727f52644e2db75fda3bf743104a";

    let public_key_bytes = hex::decode(hex_service_account_string)
        .map_err(|_| LockError::FailedToDecodeHex)
        .unwrap_or_revert();

    let service_public_key = PublicKey::from_bytes(&*public_key_bytes)
        .map_err(|_| LockError::FailedToPrepareServicePublicKey)
        .unwrap_or_revert().0;

    transfer_to_public_key(service_public_key, amount, None)
        .map_err(|_| LockError::FailedToTransferFunds)
        .unwrap_or_revert();

    runtime::call_contract::<()>(
        bridge_contract_hash,
        ENTRY_POINT_LOCK_NFT,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_DESTINATION_CHAIN => destination_chain,
            ARG_DESTINATION_USER_ADDRESS => destination_user_address,
            ARG_SOURCE_NFT_CONTRACT_ADDRESS => source_nft_contract_address,
            ARG_METADATA => metadata,
            ARG_STORAGE_ADDRESS => storage_address,
            ARG_NFT_SENDER_ADDRESS => nft_sender_address
        },
    );
}
