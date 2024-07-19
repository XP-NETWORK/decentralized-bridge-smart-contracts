// In general, data that is stored for user display may be different from the data used
// for internal functions of the smart contract. That is why we have StoreOffspringInfo.

use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


/// code hash and address of a contract
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ContractInfo {
    /// contract's code hash string
    pub code_hash: String,
    /// contract's address
    pub address: Addr,
}

/// this corresponds to RegisterOffspringInfo in factory, it is used to register
/// an offspring in the factory after the callback.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplyStorageInfo {
    /// label used when initializing offspring
    pub label: String,
    pub address: Addr,
    pub code_hash: String,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
    pub collection_code_hash: String
}
