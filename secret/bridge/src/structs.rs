// In general, data that is stored for user display may be different from the data used
// for internal functions of the smart contract. That is why we have StoreOffspringInfo.

use cosmwasm_std::{Addr, Binary};
use schemars::{JsonSchema, Map};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, JsonSchema, Serialize)]
pub struct Validator {
    pub address: Addr,
    pub added: bool,
    pub pending_reward: u128,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, JsonSchema, Serialize)]

pub struct DuplicateToOriginalContractInfo {
    pub chain: String,
    pub contract_address: String,
    pub code_hash: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, JsonSchema, Serialize)]

pub struct OriginalToDuplicateContractInfo {
    pub chain: String,
    pub contract_address: Addr,
    pub code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct State {
    pub collection_deployer: Addr,
    pub storage_deployer: Addr,
    pub validators_count: i128,
    pub self_chain: String,
    pub type_erc_721: String,
    pub type_erc_1155: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct SignerAndSignature {
    pub signer_address: Binary,
    pub signature: Binary,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub validators: Vec<(Binary, Addr)>,
    pub chain_type: String,
    // pub collection_deployer: Addr,
    // pub storage_deployer: Addr,
    pub storage_label: String,
    pub collection_label: String,
    pub collection721_code_info: CodeInfo,
    pub storage721_code_info: CodeInfo,
    pub collection1155_code_info: CodeInfo,
    pub storage1155_code_info: CodeInfo,
    pub collection_deployer_code_info: CodeInfo,
    pub storage_deployer_code_info: CodeInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct AddValidatorMsg {
    pub validator: (Binary, Addr),
    pub signatures: Vec<SignerAndSignature>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct ClaimValidatorRewardsMsg {
    pub validator: Binary,
    pub signatures: Vec<SignerAndSignature>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Lock721Msg {
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: Addr,
    pub collection_code_info: CodeInfo,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Lock1155Msg {
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: Addr,
    pub collection_code_info: CodeInfo,
    pub token_id: String,
    pub token_amount: u128,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ClaimMsg {
    pub data: ClaimData,
    pub signatures: Vec<SignerAndSignature>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct VerifyMsg {
    pub user: Binary,
    pub message: [u8; 32],
    pub signature: Binary,
    pub msg_as_bindary: Binary,
    pub claim_data: [u8; 32],
    pub claim_data_as_binary: ClaimData,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct TransferToStorage721Msg {
    pub storage_mapping721: Map<String, Map<String, Addr>>,
    pub source_nft_contract_address: Addr,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Royalty {
    /// address to send royalties to
    pub recipient: String,
    /// royalty rate
    pub rate: u16,
}

/// all royalty information
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct RoyaltyInfo {
    /// decimal places in royalty rates
    pub decimal_places_in_rates: u8,
    /// list of royalties
    pub royalties: Vec<Royalty>,
}

/// Info needed to instantiate an offspring
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CodeInfo {
    /// code id of the stored offspring contract
    pub code_id: u64,
    /// code hash of the stored offspring contract
    pub code_hash: String,
}

/// code hash and address of a contract
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ContractInfo {
    /// contract's code hash string
    pub code_hash: String,
    /// contract's address
    pub address: Addr,
}

/// this corresponds to RegisterOffspringInfo in factory, it is used to register
/// an offspring in the factory after the callback.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplyStorageInfo {
    /// label used when initializing offspring
    pub label: String,
    pub address: Addr,
    pub code_hash: String,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
    pub collection_code_hash: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplyStorageDeployerInfo {
    pub address: Addr,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplyCollectionInfo {
    /// label used when initializing offspring
    pub label: String,
    pub owner: Addr,
    pub address: Addr,
    pub code_hash: String,
    pub source_nft_contract_address: String,
    pub source_chain: String,
    pub transaction_hash: String,
    pub destination_user_address: Addr,
    pub token_id: String,
    pub token_amount: u128,
    pub royalty: u16,
    pub royalty_receiver: Addr,
    pub metadata: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplyCollectionDeployerInfo {
    pub address: Addr,
}
