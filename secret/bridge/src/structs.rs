// In general, data that is stored for user display may be different from the data used
// for internal functions of the smart contract. That is why we have StoreOffspringInfo.

use common::CodeInfo;
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
pub struct BridgeInstantiateMsg {
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
    pub metadata_uri: String
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Lock1155Msg {
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: Addr,
    pub collection_code_info: CodeInfo,
    pub token_id: String,
    pub token_amount: u128,
    pub metadata_uri: String
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
    pub lock_tx_chain: String
}

impl ClaimData {
    pub fn concat_all_fields(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
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
            self.fee,
            self.lock_tx_chain
        )
    }
}