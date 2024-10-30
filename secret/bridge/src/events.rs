//! Events emitted by the Bridge Contract
use std::convert::TryInto;

use cosmwasm_std::{Addr, Attribute, StdError, StdResult, Uint256};

use serde::Serialize;

/// Converts an event to [LogAttribute]
fn to_log_attr<T: Serialize>(name: &str, e: &T) -> StdResult<Attribute> {
    let ser =
        serde_json_wasm::to_string(e).map_err(|e| StdError::serialize_err("serde-json-wasm", e))?;

    Ok(Attribute::new_plaintext(name, ser))
    // Ok(plaintext_log(name, ser))
}

/// implements conversion from event to LogAttribute
macro_rules! bridge_event {
    ($ev:ident) => {
        impl TryInto<Attribute> for $ev {
            type Error = StdError;

            fn try_into(self) -> StdResult<Attribute> {
                to_log_attr(stringify!($ev), &self)
            }
        }
    };
}

#[derive(Debug, Serialize)]
pub struct AddNewValidatorEventInfo {
    pub validator: Addr,
}
bridge_event!(AddNewValidatorEventInfo);

impl AddNewValidatorEventInfo {
    pub fn new(validator: Addr) -> Self {
        Self { validator }
    }
}

#[derive(Debug, Serialize)]
pub struct RewardValidatorEventInfo {
    pub validator: Addr,
}
bridge_event!(RewardValidatorEventInfo);

impl RewardValidatorEventInfo {
    pub fn new(validator: Addr) -> Self {
        Self { validator }
    }
}

#[derive(Debug, Serialize)]
pub struct LockedEventInfo {
    pub token_id: String,
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: String,
    pub token_amount: u128,
    pub nft_type: String,
    pub source_chain: String,
    pub metadata_uri: String
}
bridge_event!(LockedEventInfo);

impl LockedEventInfo {
    pub fn new(
        token_id: String,
        destination_chain: String,
        destination_user_address: String,
        source_nft_contract_address: String,
        token_amount: u128,
        nft_type: String,
        source_chain: String,
        metadata_uri: String
    ) -> Self {
        Self {
            token_id,
            destination_chain,
            destination_user_address,
            source_nft_contract_address,
            token_amount,
            nft_type,
            source_chain,
            metadata_uri
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UnLock721EventInfo {
    pub to: Addr,
    pub token_id: String,
    pub contract_address: String,
}
bridge_event!(UnLock721EventInfo);

impl UnLock721EventInfo {
    pub fn new(to: Addr, token_id: String, contract_address: String) -> Self {
        Self {
            to,
            token_id,
            contract_address,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UnLock1155EventInfo {
    pub to: Addr,
    pub token_id: String,
    pub contract_address: String,
    pub amount: Uint256,
}
bridge_event!(UnLock1155EventInfo);

impl UnLock1155EventInfo {
    pub fn new(to: Addr, token_id: String, contract_address: String, amount: Uint256) -> Self {
        Self {
            to,
            token_id,
            contract_address,
            amount,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Claimed721EventInfo {
    pub lock_tx_chain: String,
    pub source_chain: String,
    pub transaction_hash: String,
    pub nft_contract: String,
    pub token_id: String,
}
bridge_event!(Claimed721EventInfo);

impl Claimed721EventInfo {
    pub fn new(lock_tx_chain: String, source_chain: String, transaction_hash: String, nft_contract: String, token_id: String) -> Self {
        Self {
            lock_tx_chain,
            source_chain,
            transaction_hash,
            nft_contract,
            token_id,
        }
    }
}


#[derive(Debug, Serialize)]
pub struct Claimed1155EventInfo {
    pub lock_tx_chain: String,
    pub source_chain: String,
    pub transaction_hash: String,
    pub nft_contract: String,
    pub token_id: String,
    pub amount: u128
}
bridge_event!(Claimed1155EventInfo);

impl Claimed1155EventInfo {
    pub fn new(lock_tx_chain: String, source_chain: String, transaction_hash: String, nft_contract: String, token_id: String, amount: u128) -> Self {
        Self {
            lock_tx_chain,
            source_chain,
            transaction_hash,
            nft_contract,
            token_id,
            amount
        }
    }
}
