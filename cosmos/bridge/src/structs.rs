// In general, data that is stored for user display may be different from the data used
// for internal functions of the smart contract. That is why we have StoreOffspringInfo.

use cosmwasm_schema::{cw_serde, schemars::Map};
use cosmwasm_std::{Addr, Binary};

#[cw_serde]
pub struct Validator {
    pub address: Addr,
    pub added: bool,
    pub pending_reward: u128,
}
#[cw_serde]

pub struct DuplicateToOriginalContractInfo {
    pub chain: String,
    pub contract_address: String,
}
#[cw_serde]

pub struct OriginalToDuplicateContractInfo {
    pub chain: String,
    pub contract_address: Addr,
}

#[cw_serde]
pub struct State {
    pub collection_deployer: Addr,
    pub storage_deployer: Addr,
    pub validators_count: i128,
    pub self_chain: String,
    pub type_erc_721: String,
    pub type_erc_1155: String,
}
#[cw_serde]
pub struct SignerAndSignature {
    pub signer_address: Binary,
    pub signature: Binary,
}

#[cw_serde]
pub struct BridgeInstantiateMsg {
    pub validators: Vec<(Binary, Addr)>,
    pub chain_type: String,
    // pub collection_deployer: Addr,
    // pub storage_deployer: Addr,
    pub storage_label: String,
    pub collection_label: String,
    pub collection721_code_id: u64,
    pub storage721_code_id: u64,
    pub collection_deployer_code_id: u64,
    pub storage_deployer_code_id: u64,
}
#[cw_serde]
pub struct AddValidatorMsg {
    pub validator: (Binary, Addr),
    pub signatures: Vec<SignerAndSignature>,
}
#[cw_serde]
pub struct ClaimValidatorRewardsMsg {
    pub validator: Binary,
    pub signatures: Vec<SignerAndSignature>,
}

#[cw_serde]
pub struct Lock721Msg {
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: String,
    pub collection_code_id: u64,
    pub token_id: String,
}

#[cw_serde]
pub struct Lock1155Msg {
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: Addr,
    pub collection_code_id: u64,
    pub token_id: String,
    pub token_amount: u128,
}

#[cw_serde]
pub struct ClaimMsg {
    pub data: ClaimData,
    pub signatures: Vec<SignerAndSignature>,
}

#[cw_serde]
pub struct VerifyMsg {
    pub user: Binary,
    pub message: [u8; 32],
    pub signature: Binary,
    pub msg_as_bindary: Binary,
    pub claim_data: [u8; 32],
    pub claim_data_as_binary: ClaimData,
}
#[cw_serde]
pub struct TransferToStorage721Msg {
    pub storage_mapping721: Map<String, Map<String, Addr>>,
    pub source_nft_contract_address: Addr,
    pub token_id: String,
}
#[cw_serde]
pub struct ClaimData {
    pub token_id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub destination_user_address: Addr,
    pub source_nft_contract_address: String,
    pub name: String,
    pub symbol: String,
    pub royalty: u16,
    pub royalty_receiver: Addr,
    pub metadata: String,
    pub transaction_hash: String,
    pub token_amount: u128,
    pub nft_type: String,
    pub fee: u128,
}

impl ClaimData {
    pub fn concat_all_fields(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            self.token_id,
            self.source_chain,
            self.destination_chain,
            self.destination_user_address,
            self.source_nft_contract_address,
            self.name,
            self.symbol,
            self.royalty,
            self.royalty_receiver,
            self.metadata,
            self.transaction_hash,
            self.token_amount,
            self.nft_type,
            self.fee
        )
    }
}

/// data for a single royalty
#[cw_serde]
pub struct Royalty {
    /// address to send royalties to
    pub recipient: String,
    /// royalty rate
    pub rate: u16,
}

/// all royalty information
#[cw_serde]
pub struct RoyaltyInfo {
    /// decimal places in royalty rates
    pub decimal_places_in_rates: u8,
    /// list of royalties
    pub royalties: Vec<Royalty>,
}

/// this corresponds to RegisterOffspringInfo in factory, it is used to register
/// an offspring in the factory after the callback.
#[cw_serde]
pub struct ReplyStorageInfo {
    /// label used when initializing offspring
    pub label: String,
    pub address: Addr,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
}

#[cw_serde]
pub struct ReplyStorageDeployerInfo {
    pub address: Addr,
}

#[cw_serde]
pub struct ReplyCollectionInfo {
    /// label used when initializing offspring
    pub label: String,
    pub owner: Addr,
    pub address: Addr,
    pub source_nft_contract_address: String,
    pub source_chain: String,
    pub destination_user_address: Addr,
    pub token_id: String,
    pub token_amount: u128,
    pub royalty: u16,
    pub royalty_receiver: Addr,
    pub metadata: String,
    pub transaction_hash: String,
}

#[cw_serde]
pub struct ReplyCollectionDeployerInfo {
    pub address: Addr,
}
