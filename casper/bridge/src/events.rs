use alloc::string::{String, ToString};
use casper_event_standard::Event;
use casper_types::{PublicKey, U256, U512};
use casper_types::account::AccountHash;
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
    pub fn new(token_id: TokenIdentifier,
               destination_chain: String,
               destination_user_address: String,
               source_nft_contract_address: String,
               token_amount: U256,
               nft_type: String,
               source_chain: String,
               metadata_uri: String) -> Self {
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
pub struct UnLock {
    pub to: String,
    pub token_id: String,
    pub contract_address: String,
}
impl UnLock {
    pub fn new(to: String,
               token_id: String,
               contract_address: String) -> Self {
        Self {
            to,
            token_id,
            contract_address,
        }
    }
}
#[derive(Event, Debug, PartialEq, Eq)]
pub struct Claimed {
    pub lock_tx_chain: String,
    pub source_chain: String,
    pub transaction_hash: String,
    pub nft_contract: String,
    pub token_id: String,
}
impl Claimed {
    pub fn new(lock_tx_chain: String,
               source_chain: String,
               transaction_hash: String,
               nft_contract: String,
               token_id: String,
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
    pub token_id: TokenIdentifier,
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: String,
    pub metadata: String,
    pub sender_address: String,
}
impl DeployStorage {
    pub fn new(
        token_id: TokenIdentifier,
        destination_chain: String,
        destination_user_address: String,
        source_nft_contract_address: String,
        metadata: String,
        sender_address: String,
    ) -> Self {
        Self {
            token_id,
            destination_chain,
            destination_user_address,
            source_nft_contract_address,
            metadata,
            sender_address,
        }
    }
}
#[derive(Event, Debug, PartialEq, Eq)]
pub struct DeployCollection {
    pub token_id: TokenIdentifier,
    pub source_chain: String,
    pub destination_chain: String,
    pub destination_user_address: AccountHash,
    pub source_nft_contract_address: String,
    pub name: String,
    pub symbol: String,
    pub royalty: U512,
    pub royalty_receiver: AccountHash,
    pub metadata: String,
    pub transaction_hash: String,
    pub token_amount: U512,
    pub nft_type: String,
    pub fee: U512,
    pub lock_tx_chain: String,
}
impl DeployCollection {
    pub fn new(
        token_id: TokenIdentifier,
        source_chain: String,
        destination_chain: String,
        destination_user_address: AccountHash,
        source_nft_contract_address: String,
        name: String,
        symbol: String,
        royalty: U512,
        royalty_receiver: AccountHash,
        metadata: String,
        transaction_hash: String,
        token_amount: U512,
        nft_type: String,
        fee: U512,
        lock_tx_chain: String,
    ) -> Self {
        Self {
            token_id,
            source_chain,
            destination_chain,
            destination_user_address,
            source_nft_contract_address,
            name,
            symbol,
            royalty,
            royalty_receiver,
            metadata,
            transaction_hash,
            token_amount,
            nft_type,
            fee,
            lock_tx_chain,
        }
    }
}