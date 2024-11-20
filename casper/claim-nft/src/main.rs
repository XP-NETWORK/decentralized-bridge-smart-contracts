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
use crate::errors::ClaimError;
const ARG_BRIDGE_CONTRACT_HASH: &str = "bridge_contract";
const ARG_TOKEN_ID: &str = "token_id_arg";
const ARG_SOURCE_CHAIN: &str = "source_chain_arg";
const ARG_DESTINATION_CHAIN: &str = "destination_chain_arg";
const ARG_DESTINATION_USER_ADDRESS: &str = "destination_user_address_arg";
const ARG_SOURCE_NFT_CONTRACT_ADDRESS: &str = "source_nft_contract_address_arg";
const ARG_NAME: &str = "name_arg";
const ARG_SYMBOL: &str = "symbol_arg";
const ARG_ROYALTY: &str = "royalty_arg";
const ARG_ROYALTY_RECEIVER: &str = "royalty_receiver_arg";
const ARG_METADATA: &str = "metadata_arg";
const ARG_TRANSACTION_HASH: &str = "transaction_hash_arg";
const ARG_TOKEN_AMOUNT: &str = "token_amount_arg";
const ARG_NFT_TYPE: &str = "nft_type_arg";
const ARG_FEE: &str = "fee_arg";
const ARG_LOCK_TX_CHAIN: &str = "lock_tx_chain_arg";
const ARG_COLLECTION_ADDRESS: &str = "collection_address_arg";
const ARG_AMOUNT: &str = "amount";

#[no_mangle]
pub extern "C" fn call() {
    let bridge_contract_hash: ContractHash = runtime::get_named_arg::<ContractHash>(ARG_BRIDGE_CONTRACT_HASH);
    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);
    let source_chain: String = runtime::get_named_arg(ARG_SOURCE_CHAIN);
    let destination_chain: String = runtime::get_named_arg(ARG_DESTINATION_CHAIN);
    let destination_user_address: AccountHash = runtime::get_named_arg(ARG_DESTINATION_USER_ADDRESS);
    let source_nft_contract_address: ContractHash = runtime::get_named_arg(ARG_SOURCE_NFT_CONTRACT_ADDRESS);
    let name: String = runtime::get_named_arg(ARG_NAME);
    let symbol: String = runtime::get_named_arg(ARG_SYMBOL);
    let royalty: U512 = runtime::get_named_arg(ARG_ROYALTY);
    let royalty_receiver: AccountHash = runtime::get_named_arg(ARG_ROYALTY_RECEIVER);
    let metadata: String = runtime::get_named_arg(ARG_METADATA);
    let transaction_hash: String = runtime::get_named_arg(ARG_TRANSACTION_HASH);
    let token_amount: U512 = runtime::get_named_arg(ARG_TOKEN_AMOUNT);
    let nft_type: String = runtime::get_named_arg(ARG_NFT_TYPE);
    let fee: U512 = runtime::get_named_arg(ARG_FEE);
    let lock_tx_chain: String = runtime::get_named_arg(ARG_LOCK_TX_CHAIN);
    let collection_address: Option<ContractHash> = runtime::get_named_arg(ARG_COLLECTION_ADDRESS);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    const ENTRY_POINT_CLAIM_NFT: &str = "claim";

    let amount_for_service: U512 = amount - U512::from(40).pow(U512::from(9));
    let amount_for_bridge: U512 = U512::from(40).pow(U512::from(9));

    let hex_service_account_string = "01e8a472f805d3c4f01644cbd75bc467a73d14727f52644e2db75fda3bf743104a";
    let hex_bridge_account_string = "4e8b075dfbf361e7bcd53a9f72313402e7ba457c36dd2ecaf6703115c51d6126";

    let public_key_bytes = hex::decode(hex_service_account_string)
        .map_err(|_| ClaimError::FailedToDecodeHex)
        .unwrap_or_revert();

    let service_public_key = PublicKey::from_bytes(&*public_key_bytes)
        .map_err(|_| ClaimError::FailedToPrepareServicePublicKey)
        .unwrap_or_revert().0;

    transfer_to_public_key(service_public_key, amount_for_service, None)
        .map_err(|_| ClaimError::FailedToTransferFunds)
        .unwrap_or_revert();

    let bridge_public_key_bytes = hex::decode(hex_bridge_account_string)
        .map_err(|_| ClaimError::FailedToDecodeHex)
        .unwrap_or_revert();

    let bridge_public_key = PublicKey::from_bytes(&*bridge_public_key_bytes)
        .map_err(|_| ClaimError::FailedToPrepareServicePublicKey)
        .unwrap_or_revert().0;

    transfer_to_public_key(bridge_public_key, amount_for_bridge, None)
        .map_err(|_| ClaimError::FailedToTransferFunds)
        .unwrap_or_revert();

    runtime::call_contract::<()>(
        bridge_contract_hash,
        ENTRY_POINT_CLAIM_NFT,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_CHAIN => source_chain,
            ARG_DESTINATION_CHAIN => destination_chain,
            ARG_DESTINATION_USER_ADDRESS => destination_user_address,
            ARG_SOURCE_NFT_CONTRACT_ADDRESS => source_nft_contract_address,
            ARG_NAME => name,
            ARG_SYMBOL => symbol,
            ARG_ROYALTY => royalty,
            ARG_ROYALTY_RECEIVER => royalty_receiver,
            ARG_METADATA => metadata,
            ARG_TRANSACTION_HASH => transaction_hash,
            ARG_TOKEN_AMOUNT => token_amount,
            ARG_NFT_TYPE => nft_type,
            ARG_FEE => fee,
            ARG_LOCK_TX_CHAIN => lock_tx_chain,
            ARG_COLLECTION_ADDRESS => collection_address,
        },
    );
}
