use cosmwasm_std::StdError;

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum CollectionFactoryContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Unexpected reply id :{id}")]
    UnexpectedReplyId { id: u64 },

    #[error("Got a custome error :{0}")]
    CustomError(String),
}
