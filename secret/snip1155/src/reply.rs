use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};

/// this corresponds to ReplyOffspringInfo in factory, it is used to register
/// an offspring in the factory after the callback.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplyCollectionInfo {
    /// label used when initializing offspring
    pub label: String,
    pub owner: Addr,
    pub address: Addr,
    pub code_hash: String,
    pub source_nft_contract_address: String,
    pub source_chain: String,
    pub destination_user_address: Addr,
    pub token_id: String,
    pub token_amount: u128,
    pub royalty: u16,
    pub royalty_receiver: Addr,
    pub metadata: String,
    pub transaction_hash: String,
    pub lock_tx_chain: String
}
