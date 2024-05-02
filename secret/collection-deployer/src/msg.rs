use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{offspring_msg::CurateTokenId, structs::CodeInfo};

/// Instantiation message
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    /// collection code info
    pub collection721_code_info: CodeInfo,
    pub collection1155_code_info: CodeInfo,
}

/// Handle messages
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateCollection721 {
        owner: String,
        name: String,
        symbol: String,
        source_nft_contract_address: String,
        source_chain: String,
        destination_user_address: Addr,
        token_id: String,
        token_amount: u128,
        royalty: u16,
        royalty_receiver: Addr,
        metadata: String,
        transaction_hash: String
    },
    CreateCollection1155 {
        // owner: String,
        name: String,
        symbol: String,
        // source_nft_contract_address: String,
        // source_chain: String,
        // destination_user_address: Addr,
        // token_id: String,
        // token_amount: u128,
        // royalty: u16,
        // royalty_receiver: Addr,
        // metadata: String,
        has_admin: bool,
        admin: Option<Addr>,
        curators: Vec<Addr>,
        initial_tokens: Vec<CurateTokenId>,
        entropy: String,
        label: String,
        source_nft_contract_address: String,
        source_chain: String,
        destination_user_address: Addr,
        token_id: String,
        token_amount: u128,
        royalty: u16,
        royalty_receiver: Addr,
        metadata: String,
        transaction_hash: String
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
