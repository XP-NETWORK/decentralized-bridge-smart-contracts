use common::CodeInfo;
use cosmwasm_std::Addr;
use secret_toolkit::utils::InitCallback;
use serde::{Deserialize, Serialize};

use crate::state::BLOCK_SIZE;

#[derive(Serialize, Deserialize)]
pub struct StorageInstantiateMsg {
    pub collection_address: Addr,
    pub owner: Addr,
    pub collection_code_info: CodeInfo,
    pub is_original: bool,
    pub token_id: String,
}

impl InitCallback for StorageInstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
