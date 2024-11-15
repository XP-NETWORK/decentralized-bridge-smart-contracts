use alloc::string::{String, ToString};
use casper_event_standard::Event;
use casper_types::{bytesrepr::{FromBytes, ToBytes}, CLType, CLTyped, ContractHash, PublicKey, U256};
use crate::external::xp_nft::TokenIdentifier;

impl CLTyped for TokenIdentifier {
    fn cl_type() -> casper_types::CLType {
        CLType::String
    }
}

impl FromBytes for TokenIdentifier {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (tid, remainder) = String::from_bytes(bytes)?;
        match tid.parse::<u64>() {
            Ok(e) => Ok((TokenIdentifier::Index(e), remainder)),
            Err(_) => Ok((TokenIdentifier::Hash(tid), remainder)),
        }
    }
}
impl ToBytes for TokenIdentifier {
    fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, casper_types::bytesrepr::Error> {
        match self {
            TokenIdentifier::Index(index) => index.to_string().to_bytes(),
            TokenIdentifier::Hash(hash) => hash.to_bytes(),
        }
    }

    fn serialized_length(&self) -> usize {
        match self {
            TokenIdentifier::Index(e) => e.to_string().serialized_length(),
            TokenIdentifier::Hash(h) => h.serialized_length(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct AddNewValidator {
    pub public_key: String,
}

impl AddNewValidator {
    pub fn new(public_key: PublicKey) -> Self {
        Self {
            public_key: public_key.to_string(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Locked {
    pub token_id: U256,
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: ContractHash,
    pub token_amount: U256,
    pub nft_type: String,
    pub chain: String,
    pub metadata_uri: String,
}

impl Locked {
    pub fn new(token_id: U256,
               destination_chain: String,
               destination_user_address: String,
               source_nft_contract_address: ContractHash,
               token_amount: U256,
               nft_type: String,
               chain: String,
               metadata_uri: String) -> Self {
        Self {
            token_id,
            destination_chain,
            destination_user_address,
            source_nft_contract_address,
            token_amount,
            nft_type,
            chain,
            metadata_uri,
        }
    }
}