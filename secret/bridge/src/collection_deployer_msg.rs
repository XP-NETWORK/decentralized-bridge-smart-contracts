use cosmwasm_std::{Addr, Uint256};
use schemars::JsonSchema;
use secret_toolkit::{
    snip721::Metadata,
    utils::{HandleCallback, InitCallback},
};
use serde::{Deserialize, Serialize};

use crate::{state::BLOCK_SIZE, structs::CodeInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenIdBalance {
    /// For BurnToken, `address` needs to be the owner's address. This design decision is
    /// to allow `BurnToken` to apply to other addresses, possible in the additional
    /// specifications
    pub address: Addr,
    pub amount: Uint256,
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
impl TknConfig {
    /// Combines variables in the TknConfig enum into a single struct for easier handling in contract logic.
    pub fn flatten(&self) -> TknConfigFlat {
        match self {
            TknConfig::Fungible {
                minters,
                decimals,
                public_total_supply,
                enable_mint,
                enable_burn,
                minter_may_update_metadata,
            } => {
                TknConfigFlat {
                    is_nft: false,
                    minters: minters.clone(),
                    decimals: *decimals,
                    public_total_supply: *public_total_supply,
                    owner_is_public: false,
                    enable_mint: *enable_mint,
                    enable_burn: *enable_burn,
                    minter_may_update_metadata: *minter_may_update_metadata,
                    // there can be multiple owners, so owners cannot update metadata
                    owner_may_update_metadata: false,
                }
            }
            TknConfig::Nft {
                minters,
                public_total_supply,
                owner_is_public,
                enable_burn,
                owner_may_update_metadata,
                minter_may_update_metadata,
            } => {
                TknConfigFlat {
                    is_nft: true,
                    // NFTs' minters cannot mint additional tokens, but may be able to change metadata
                    minters: minters.clone(),
                    decimals: 0_u8,
                    public_total_supply: *public_total_supply,
                    owner_is_public: *owner_is_public,
                    // NFT can be minted only once using `CurateTokenIds`
                    enable_mint: false,
                    enable_burn: *enable_burn,
                    minter_may_update_metadata: *minter_may_update_metadata,
                    owner_may_update_metadata: *owner_may_update_metadata,
                }
            }
        }
    }
}

/// Constructed from input enum `TknConfig`. Flattened for easier handling in contract logic
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TknConfigFlat {
    pub is_nft: bool,
    pub minters: Vec<Addr>,
    pub decimals: u8,
    pub public_total_supply: bool,
    pub owner_is_public: bool,
    pub enable_mint: bool,
    pub enable_burn: bool,
    pub minter_may_update_metadata: bool,
    pub owner_may_update_metadata: bool,
}

impl TknConfigFlat {
    pub fn to_enum(&self) -> TknConfig {
        match self.is_nft {
            true => TknConfig::Nft {
                minters: self.minters.clone(),
                public_total_supply: self.public_total_supply,
                owner_is_public: self.owner_is_public,
                enable_burn: self.enable_burn,
                owner_may_update_metadata: self.owner_may_update_metadata,
                minter_may_update_metadata: self.minter_may_update_metadata,
            },
            false => TknConfig::Fungible {
                minters: self.minters.clone(),
                decimals: self.decimals,
                public_total_supply: self.public_total_supply,
                enable_mint: self.enable_mint,
                enable_burn: self.enable_burn,
                minter_may_update_metadata: self.minter_may_update_metadata,
            },
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurateTokenId {
    pub token_info: TokenInfoMsg,
    pub balances: Vec<TokenIdBalance>,
}

#[derive(Serialize)]
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
        /// if `admin` == `None` && `has_admin` == `true`, the instantiator will be admin
        /// if `has_admin` == `false`, this field will be ignore (ie: there will be no admin)
        admin: Option<Addr>,
        /// sets initial list of curators, which can create new token_ids
        curators: Vec<Addr>,
        /// curates initial list of tokens
        initial_tokens: Vec<CurateTokenId>,
        /// for `create_viewing_key` function
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

impl HandleCallback for CollectionDeployerExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize)]
pub struct InstantiateCollectionDeployer {
    pub collection721_code_info: CodeInfo,
    pub collection1155_code_info: CodeInfo,
}

impl InitCallback for InstantiateCollectionDeployer {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

/// message sent my instantiator and curators for a specific `token_id`'s token info
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LbPair {
    pub name: String,
    pub symbol: String,
    pub lb_pair_address: Addr,
    pub decimals: u8,
}
