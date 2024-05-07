use cosmwasm_std::Addr;
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback};
use serde::{Deserialize, Serialize};
use common::CodeInfo;

use crate::state::BLOCK_SIZE;

/// Instantiation message
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct StorageDeployerInstantiateMsg {
    /// collection code info
    pub storage721_code_info: CodeInfo,
    pub storage1155_code_info: CodeInfo,
}

/// Handle messages
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StorageDeployerExecuteMsg {
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
pub enum StorageDeployerHandleAnswer {
    /// generic status response
    Status {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

impl HandleCallback for StorageDeployerExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl InitCallback for StorageDeployerInstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}