use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId, NearSchema,
};


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema, Hash)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimData {
    pub token_id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub destination_user_address: AccountId,
    pub source_nft_contract_address: String,
    pub name: String,
    pub symbol: String,
    pub royalty: u16,
    pub royalty_receiver: AccountId,
    pub metadata: String,
    pub transaction_hash: String,
    pub token_amount: u128,
    pub nft_type: String,
    pub fee: u128,
    pub lock_tx_chain: String,
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct SignerAndSignature {
    pub signer: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct ContractInfo {
    pub chain: String,
    pub contract_address: String,
}
