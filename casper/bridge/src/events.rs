use alloc::string::{String, ToString};
use casper_event_standard::Event;
use casper_types::{PublicKey, U256, U512};
use common::collection::TokenIdentifier;

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
pub struct BlackListValidator {
    pub public_key: String,
}
impl BlackListValidator {
    pub fn new(public_key: PublicKey) -> Self {
        Self {
            public_key: public_key.to_string(),
        }
    }
}
#[derive(Event, Debug, PartialEq, Eq)]
pub struct RewardValidator {
    pub public_key: String,
}
impl RewardValidator {
    pub fn new(public_key: PublicKey) -> Self {
        Self {
            public_key: public_key.to_string(),
        }
    }
}
#[derive(Event, Debug, PartialEq, Eq)]
pub struct Locked {
    pub token_id: TokenIdentifier,
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: String,
    pub token_amount: U256,
    pub nft_type: String,
    pub source_chain: String,
    pub metadata_uri: String,
}
impl Locked {
    pub fn new(
        token_id: TokenIdentifier,
        destination_chain: String,
        destination_user_address: String,
        source_nft_contract_address: String,
        token_amount: U256,
        nft_type: String,
        source_chain: String,
        metadata_uri: String,
    ) -> Self {
        Self {
            token_id,
            destination_chain,
            destination_user_address,
            source_nft_contract_address,
            token_amount,
            nft_type,
            source_chain,
            metadata_uri,
        }
    }
}
#[derive(Event, Debug, PartialEq, Eq)]
pub struct Claimed {
    pub lock_tx_chain: String,
    pub source_chain: String,
    pub transaction_hash: String,
    pub nft_contract: String,
    pub token_id: TokenIdentifier,
}
impl Claimed {
    pub fn new(
        lock_tx_chain: String,
        source_chain: String,
        transaction_hash: String,
        nft_contract: String,
        token_id: TokenIdentifier,
    ) -> Self {
        Self {
            lock_tx_chain,
            source_chain,
            transaction_hash,
            nft_contract,
            token_id,
        }
    }
}
#[derive(Event, Debug, PartialEq, Eq)]
pub struct DeployStorage {
    pub source_nft_contract_address: String,
}
impl DeployStorage {
    pub fn new(source_nft_contract_address: String) -> Self {
        Self {
            source_nft_contract_address,
        }
    }
}
#[derive(Event, Debug, PartialEq, Eq)]
pub struct DeployCollection {
    pub source_chain: String,
    pub source_nft_contract_address: String,
}
impl DeployCollection {
    pub fn new(source_chain: String, source_nft_contract_address: String) -> Self {
        Self {
            source_chain,
            source_nft_contract_address,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct CollectionDeployFee {
    pub fee: U512,
}
impl CollectionDeployFee {
    pub fn new(fee: U512) -> Self {
        Self { fee }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct StorageDeployFee {
    pub fee: U512,
}
impl StorageDeployFee {
    pub fn new(fee: U512) -> Self {
        Self { fee }
    }
}
