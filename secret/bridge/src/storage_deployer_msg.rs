use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};

use secret_toolkit::utils::{HandleCallback, InitCallback};

use crate::{state::BLOCK_SIZE, structs::CodeInfo};

/// Factory handle messages to be used by offspring.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageDeployerExecuteMsg {
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
        token_amount: u128
    },
}

impl HandleCallback for StorageDeployerExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize)]
pub struct InstantiateStorageDeployer {
    pub storage721_code_info: CodeInfo,
    pub storage1155_code_info: CodeInfo,
}

impl InitCallback for InstantiateStorageDeployer {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
