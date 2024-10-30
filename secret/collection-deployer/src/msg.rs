use common::CodeInfo;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback};
use serde::{Deserialize, Serialize};
use snip1155::state::state_structs::{CurateTokenId, LbPair};

use crate::state::BLOCK_SIZE;

/// Instantiation message
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CollectionDeployerInstantiateMsg {
    /// collection code info
    pub collection721_code_info: CodeInfo,
    pub collection1155_code_info: CodeInfo,
}

/// Handle messages
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollectionDeployerExecuteMsg {
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
        transaction_hash: String,
        lock_tx_chain: String
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
        lb_pair_info: LbPair,
        label: String,
        source_nft_contract_address: String,
        source_chain: String,
        destination_user_address: Addr,
        token_id: String,
        token_amount: u128,
        royalty: u16,
        royalty_receiver: Addr,
        metadata: String,
        transaction_hash: String,
        lock_tx_chain: String
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
pub enum CollectionDeployerHandleAnswer {
    /// generic status response
    Status {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}


impl HandleCallback for CollectionDeployerExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl InitCallback for CollectionDeployerInstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
