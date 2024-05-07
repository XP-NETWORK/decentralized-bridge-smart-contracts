use common::CodeInfo;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Storage721InstantiateMsg {
    pub collection_address: Addr,
    pub owner: Addr,
    pub collection_code_info: CodeInfo,
    pub is_original: bool,
    pub token_id: String,
}

/// Executes
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Storage721ExecuteMsg {
    // DepositToken { token_id: String },
    UnLockToken { token_id: String, to: Addr },
}
