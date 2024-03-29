use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

/// this corresponds to ReplyOffspringInfo in factory, it is used to register
/// an offspring in the factory after the callback.
#[cw_serde]
pub struct StorageDeployerInfo {
    /// label used when initializing offspring
    pub label: String,
    pub address: Addr,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
}
