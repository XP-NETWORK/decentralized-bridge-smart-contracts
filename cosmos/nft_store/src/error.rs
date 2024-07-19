use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum StorageContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("You are not authorized to perform this function")]
    Unauthorized,
}
