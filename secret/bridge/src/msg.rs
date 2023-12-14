use crate::structs::{
    AddValidatorMsg, ClaimMsg, ClaimValidatorRewardsMsg, DuplicateToOriginalContractInfo,
    Lock1155Msg, Lock721Msg, OriginalToDuplicateContractInfo, Validator, VerifyMsg,
};

use cosmwasm_std::{Addr, Binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Executes
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddValidator { data: AddValidatorMsg },
    ClaimValidatorRewards { data: ClaimValidatorRewardsMsg },
    Lock721 { data: Lock721Msg },
    Lock1155 { data: Lock1155Msg },
    Claim721 { data: ClaimMsg },
    Claim1155 { data: ClaimMsg },
    VerifySig { data: VerifyMsg },
}

/// Queries
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetValidatorsCount {},
    GetValidator {
        address: Binary,
    },
    GetCollectionDeployer {},
    GetStorageDeployer {},
    GetOriginalStorage721 {
        contract_address: String,
        chain: String,
    },
    GetDuplicateStorage721 {
        contract_address: String,
        chain: String,
    },
    GetOriginalToDuplicate {
        contract_address: String,
        chain: String,
    },
    GetDuplicateToOriginal {
        contract_address: Addr,
        chain: String,
    },
}

/// responses to queries
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    ValidatorCountResponse {
        count: i128,
    },
    Validator {
        data: Option<Validator>,
    },
    CollectionDeployer {
        data: Addr,
    },
    StorageDeployer {
        data: Addr,
    },
    Storage {
        data: Option<(Addr, String)>,
    },
    OriginalToDuplicate {
        data: Option<OriginalToDuplicateContractInfo>,
    },
    DuplicateToOriginal {
        data: Option<DuplicateToOriginalContractInfo>,
    },
    // List the offspring where address is associated.
    // ListMyOffspring {
    //     /// lists of the address' active offspring
    //     #[serde(skip_serializing_if = "Option::is_none")]
    //     active: Option<Vec<StoreOffspringInfo>>,
    //     /// lists of the address' inactive offspring
    //     #[serde(skip_serializing_if = "Option::is_none")]
    //     inactive: Option<Vec<StoreOffspringInfo>>,
    // },
    // /// List active offspring
    // ListActiveOffspring {
    //     /// active offspring
    //     active: Vec<StoreOffspringInfo>,
    // },
    // /// List inactive offspring in no particular order
    // ListInactiveOffspring {
    //     /// inactive offspring in no particular order
    //     inactive: Vec<StoreOffspringInfo>,
    // },
    // /// Viewing Key Error
    // ViewingKeyError { error: String },
    // /// result of authenticating address/key pair
    // IsKeyValid { is_valid: bool },
    // /// result of authenticating a permit
    // IsPermitValid {
    //     is_valid: bool,
    //     /// address of the permit signer if the permit was valid
    //     #[serde(skip_serializing_if = "Option::is_none")]
    //     address: Option<Addr>,
    // },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct ValidatorCountResponse {
    pub count: i128,
}

// #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum ExecuteMsg {
//     Increment {},
//     Reset { count: i32 },
// }

// #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum QueryMsg {
//     // GetCount returns the current count as a json-encoded number
//     GetCount {},
// }

// // We define a custom struct for each query response
// #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
// pub struct CountResponse {
//     pub count: i32,
// }
