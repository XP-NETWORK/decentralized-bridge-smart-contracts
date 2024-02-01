use cosmwasm_std::Addr;
use serde::Serialize;

/// this corresponds to ReplyOffspringInfo in factory, it is used to register
/// an offspring in the factory after the callback.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StorageDeployerInfo {
    /// label used when initializing offspring
    pub label: String,
    pub address: Addr,
    pub code_hash: String,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
    pub collection_code_hash: String
}
