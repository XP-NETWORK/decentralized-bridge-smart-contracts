use near_sdk::serde::{Deserialize, Serialize};

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
    ValidatorBlacklisted(ValidatorBlacklisted)
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NewValidatorAdded {
    pub validator: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorBlacklisted {
    pub validator: String,
}