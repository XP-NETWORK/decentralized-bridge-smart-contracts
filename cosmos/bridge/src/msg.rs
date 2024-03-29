use crate::structs::{
    AddValidatorMsg, ClaimMsg, ClaimValidatorRewardsMsg, DuplicateToOriginalContractInfo,
    Lock721Msg, OriginalToDuplicateContractInfo, Validator, VerifyMsg,
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};

/// Executes
#[cw_serde]
pub enum BridgeExecuteMsg {
    AddValidator { data: AddValidatorMsg },
    ClaimValidatorRewards { data: ClaimValidatorRewardsMsg },
    Lock721 { data: Lock721Msg },
    Claim721 { data: ClaimMsg },
    VerifySig { data: VerifyMsg },
}

/// Queries
#[cw_serde]
#[derive(QueryResponses)]
pub enum BridgeQueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetValidatorCountResponse)]
    GetValidatorsCount {},
    #[returns(GetValidatorResponse)]
    GetValidator { address: Binary },
    #[returns(GetCollectionDeployerResponse)]
    GetCollectionDeployer {},
    #[returns(GetStorageDeployerResponse)]
    GetStorageDeployer {},
    #[returns(GetStorageResponse)]
    GetOriginalStorage721 {
        contract_address: String,
        chain: String,
    },
    #[returns(GetStorageResponse)]
    GetDuplicateStorage721 {
        contract_address: String,
        chain: String,
    },
    #[returns(GetOriginalToDuplicateResponse)]
    GetOriginalToDuplicate {
        contract_address: String,
        chain: String,
    },
    #[returns(GetDuplicateToOriginalResponse)]
    GetDuplicateToOriginal {
        contract_address: Addr,
        chain: String,
    },
}

#[cw_serde]
pub struct GetValidatorCountResponse {
    pub count: i128,
}

#[cw_serde]
pub struct GetValidatorResponse {
    pub data: Option<Validator>,
}

#[cw_serde]
pub struct GetCollectionDeployerResponse {
    pub data: Addr,
}

#[cw_serde]
pub struct GetStorageDeployerResponse {
    pub data: Addr,
}

#[cw_serde]
pub struct GetStorageResponse {
    pub data: Option<Addr>,
}

#[cw_serde]
pub struct GetOriginalToDuplicateResponse {
    pub data: Option<OriginalToDuplicateContractInfo>,
}

#[cw_serde]
pub struct GetDuplicateToOriginalResponse {
    pub data: Option<DuplicateToOriginalContractInfo>,
}
