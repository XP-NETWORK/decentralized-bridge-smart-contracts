use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::structs::CodeInfo;

/// Instantiation message
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    /// collection code info
    pub storage721_code_info: CodeInfo,
    pub storage1155_code_info: CodeInfo,
}

/// Handle messages
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// CreateOffspring will instantiate a new offspring contract
    CreateStorage721 {
        label: String,
        collection_address: Addr,
        collection_code_info: CodeInfo,
        owner: String,
        is_original: bool,
        token_id: String,
    },
    CreateStorage1155 {
        label: String,
        collection_address: Addr,
        collection_code_info: CodeInfo,
        owner: String,
        is_original: bool,
        token_id: String,
    },
}

/// success or failure response
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub enum ResponseStatus {
    Success,
    Failure,
}

/// Responses from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    /// generic status response
    Status {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}
