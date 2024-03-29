//! Events emitted by the Bridge Contract
use std::convert::TryInto;

use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{Addr, Attribute, StdError, StdResult};

/// Converts an event to [LogAttribute]
fn to_log_attr<T: Serialize>(name: &str, e: &T) -> StdResult<Attribute> {
    let ser =
        serde_json_wasm::to_string(e).map_err(|e| StdError::serialize_err("serde-json-wasm", e))?;

    Ok(Attribute::new(name, ser))
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

#[cw_serde]
pub struct AddNewValidatorEventInfo {
    pub validator: Addr,
}
bridge_event!(AddNewValidatorEventInfo);

impl AddNewValidatorEventInfo {
    pub fn new(validator: Addr) -> Self {
        Self { validator }
    }
}

#[cw_serde]
pub struct RewardValidatorEventInfo {
    pub validator: Addr,
}
bridge_event!(RewardValidatorEventInfo);

impl RewardValidatorEventInfo {
    pub fn new(validator: Addr) -> Self {
        Self { validator }
    }
}

#[cw_serde]
pub struct LockedEventInfo {
    pub token_id: String,
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: String,
    pub token_amount: u128,
    pub nft_type: String,
    pub source_chain: String,
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
    ) -> Self {
        Self {
            token_id,
            destination_chain,
            destination_user_address,
            source_nft_contract_address,
            token_amount,
            nft_type,
            source_chain,
        }
    }
}

#[cw_serde]
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

#[cw_serde]
pub struct ClaimedEventInfo {
    pub source_chain: String,
    pub transaction_hash: String,
}
bridge_event!(ClaimedEventInfo);

impl ClaimedEventInfo {
    pub fn new(source_chain: String, transaction_hash: String) -> Self {
        Self {
            source_chain,
            transaction_hash,
        }
    }
}
