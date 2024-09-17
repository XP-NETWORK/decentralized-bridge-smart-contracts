use near_sdk::{serde::{Deserialize, Serialize}, AccountId, NearToken};
use crate::external::nft_types:: TokenId;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub standard: String,
    pub version: String,

    // `flatten` to not have "event": {<EventLogVariant>} in the JSON, just have the contents of {<EventLogVariant>}.
    #[serde(flatten)]
    pub event: EventLogVariant,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
    ValidatorAdded(NewValidatorAdded),
    ValidatorBlacklisted(ValidatorBlacklisted),
    Locked(LockedEvent),
    Claimed(ClaimedEvent),
    ValidatorRewardsClaimed(ValidatorRewardsClaimed),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub  struct  ValidatorRewardsClaimed {
    pub amount: NearToken,
    pub validator: AccountId,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NewValidatorAdded {
    pub validator: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimedEvent {
    pub contract: AccountId,
    pub token_id: TokenId,
    pub transaction_hash: String,
    pub source_chain: String
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LockedEvent {
    pub token_id: String,
    pub destination_chain: String,
    pub destination_user_address: String,
    pub source_nft_contract_address: String,
    pub token_amount: u128,
    pub nft_type: String,
    pub source_chain: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorBlacklisted {
    pub validator: String,
}