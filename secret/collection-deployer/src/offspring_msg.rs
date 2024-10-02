use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::{snip721::Metadata, utils::InitCallback};
use serde::{Deserialize, Serialize};
use snip1155::state::state_structs::CurateTokenId;

use crate::state::BLOCK_SIZE;

/// Instantiation message
#[derive(Serialize, Deserialize)]
pub struct Collection721InstantiateMsg {
    /// label used when initializing offspring
    pub label: String,
    pub owner: Addr,
    pub admin: Option<String>,
    /// name of token contract
    pub name: String,
    /// token contract symbol
    pub symbol: String,
    /// entropy used for prng seed
    pub entropy: String,
    pub source_nft_contract_address: String,
    pub source_chain: String,
    pub destination_user_address: Addr,
    pub transaction_hash: String,
    pub token_id: String,
    pub token_amount: u128,
    pub royalty: u16,
    pub royalty_receiver: Addr,
    pub metadata: String,
    pub lock_tx_chain: String
}

impl InitCallback for Collection721InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenIdBalance {
    /// For BurnToken, `address` needs to be the owner's address. This design decision is
    /// to allow `BurnToken` to apply to other addresses, possible in the additional
    /// specifications
    pub address: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TknConfig {
    /// no `owner_may_update_metadata`because there can be multiple owners
    Fungible {
        minters: Vec<Addr>,
        /// Decimals play no part in the contract logic of the base specification of SNIP1155,
        /// as there are no `deposit` and `redeem` features as seen in SNIP20. The UI application
        /// has discretion in handling decimals
        decimals: u8,
        public_total_supply: bool,
        enable_mint: bool,
        enable_burn: bool,
        minter_may_update_metadata: bool,
    },
    /// no `enable_mint` option because NFT can be minted only once using `CurateTokenIds`
    Nft {
        /// NFTs' minters cannot mint additional tokens, but may be able to change metadata
        minters: Vec<Addr>,
        /// total supply can be zero if the token has been burnt
        public_total_supply: bool,
        owner_is_public: bool,
        enable_burn: bool,
        owner_may_update_metadata: bool,
        minter_may_update_metadata: bool,
    },
}

/// message sent my instantiator and curators for a specific `token_id`'s token info
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfoMsg {
    pub token_id: String,
    pub name: String,
    pub symbol: String,
    pub token_config: TknConfig,
    pub public_metadata: Option<Metadata>,
    pub private_metadata: Option<Metadata>,
}


#[derive(Serialize, Deserialize)]
pub struct Collection1155InstantiateMsg {
    pub has_admin: bool,
    pub admin: Option<Addr>,
    pub curators: Vec<Addr>,
    pub initial_tokens: Vec<CurateTokenId>,
    pub entropy: String,
    pub label: String,
    pub source_nft_contract_address: String,
    pub source_chain: String,
    pub destination_user_address: Addr,
    pub token_id: String,
    pub token_amount: u128,
    pub royalty: u16,
    pub royalty_receiver: Addr,
    pub metadata: String,
    pub transaction_hash: String,
            pub lock_tx_chain: String
}

impl InitCallback for Collection1155InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
