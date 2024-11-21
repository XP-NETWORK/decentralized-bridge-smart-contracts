#![no_std]
#![no_main]

mod errors;

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use casper_contract::{
    contract_api::{
        runtime,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_contract::contract_api::account;
use casper_contract::contract_api::system::{create_purse, transfer_from_purse_to_purse};
use casper_types::{runtime_args, ContractHash, PublicKey, RuntimeArgs, U512};
use casper_types::account::AccountHash;
use crate::errors::ClaimError;
type Sigs = (PublicKey, [u8; 64]);
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
const ARG_SIGNATURES: &str = "signatures_arg";
const ARG_AMOUNT: &str = "amount";
const ARG_SENDER_PURSE: &str = "sender_purse";
fn has_correct_fee(fee: U512, msg_value: U512) {
    if msg_value < fee {
        runtime::revert(ClaimError::FeeLessThanSentAmount);
    }
}
#[no_mangle]
pub extern "C" fn call() {
    let bridge_contract_hash = ContractHash::from_formatted_str("contract-08211cafb0698da442b68064f49c3d5e8cc303016a6ed46587d82725d42f98dc").unwrap();
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
    let signatures: Vec<Sigs> = runtime::get_named_arg(ARG_SIGNATURES);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    const ENTRY_POINT_CLAIM_NFT: &str = "claim";

    has_correct_fee(fee, amount);

    let tw_purse = create_purse();

    transfer_from_purse_to_purse(account::get_main_purse(), tw_purse, amount, None)
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
            ARG_SIGNATURES => signatures,
            ARG_SENDER_PURSE => tw_purse,
            ARG_AMOUNT => amount
        },
    );
}
