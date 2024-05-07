use common::CodeInfo;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Storage1155InstantiateMsg {
    pub collection_address: Addr,
    pub owner: Addr,
    pub collection_code_info: CodeInfo,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
}

/// Executes
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Storage1155ExecuteMsg {
    // DepositToken {
    //     token_id: String,
    //     amount: u128,
    // },
    UnLockToken {
        token_id: String,
        amount: u128,
        to: Addr,
    },
}
