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
use casper_contract::contract_api::account;
use casper_contract::contract_api::system::{create_purse, transfer_from_purse_to_purse};
use casper_types::{runtime_args, ContractHash, RuntimeArgs, U512};
const ARG_TOKEN_ID: &str = "token_id_arg";
const ARG_DESTINATION_CHAIN: &str = "destination_chain_arg";
const ARG_DESTINATION_USER_ADDRESS: &str = "destination_user_address_arg";
const ARG_SOURCE_NFT_CONTRACT_ADDRESS: &str = "source_nft_contract_address_arg";
const ARG_METADATA: &str = "metadata_arg";
const ARG_AMOUNT: &str = "amount";
const ARG_SENDER_PURSE: &str = "sender_purse";
#[no_mangle]
pub extern "C" fn call() {
    let bridge_contract_hash = ContractHash::from_formatted_str("contract-08211cafb0698da442b68064f49c3d5e8cc303016a6ed46587d82725d42f98dc").unwrap();
    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);
    let destination_chain: String = runtime::get_named_arg(ARG_DESTINATION_CHAIN);
    let destination_user_address: String = runtime::get_named_arg(ARG_DESTINATION_USER_ADDRESS);
    let source_nft_contract_address: ContractHash = runtime::get_named_arg(ARG_SOURCE_NFT_CONTRACT_ADDRESS);
    let metadata: String = runtime::get_named_arg(ARG_METADATA);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    const ENTRY_POINT_LOCK_NFT: &str = "lock";

    let tw_purse = create_purse();

    transfer_from_purse_to_purse(account::get_main_purse(), tw_purse, amount, None)
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
            ARG_SENDER_PURSE => tw_purse,
            ARG_AMOUNT => amount
        },
    );
}
