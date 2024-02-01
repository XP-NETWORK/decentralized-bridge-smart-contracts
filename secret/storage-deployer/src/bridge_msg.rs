use cosmwasm_std::Addr;
use serde::Serialize;

/// this corresponds to ReplyOffspringInfo in factory, it is used to register
/// an offspring in the factory after the callback.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BridgeInfo {
    /// label used when initializing offspring
    pub address: Addr,
}
