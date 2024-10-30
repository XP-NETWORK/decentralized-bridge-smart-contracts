use cosmwasm_std::Addr;
use secret_toolkit::utils::InitCallback;
use serde::{Deserialize, Serialize};

use crate::{state::BLOCK_SIZE, structs::CodeInfo};

#[derive(Serialize, Deserialize)]
pub struct Storage721InstantiateMsg {
    pub collection_address: Addr,
    pub owner: Addr,
    pub collection_code_info: CodeInfo,
    pub is_original: bool,
    pub token_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Storage1155InstantiateMsg {
    pub collection_address: Addr,
    pub owner: Addr,
    pub collection_code_info: CodeInfo,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
}

impl InitCallback for Storage721InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl InitCallback for Storage1155InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
