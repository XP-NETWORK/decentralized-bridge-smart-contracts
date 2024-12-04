use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum StorageError {
    AlreadyInitialized, // 0

    MissingArgumentCollection, // 1
    InvalidArgumentCollection, // 2

    MissingArgumentOwner, // 3
    InvalidArgumentOwner, // 4

    MissingArgumentSelfHash, // 5
    InvalidArgumentSelfHash, // 6

    MissingArgumentTokenId, // 7
    InvalidArgumentTokenId, // 8

    MissingArgumentTo, // 9
    InvalidArgumentTo, // 10

    FailedToGetArgBytes,  // 11
    UnexpectedKeyVariant, // 12

    MissingCollectionRef, // 13
    InvalidCollectionRef, // 14

    MissingSelfHashRef, // 15
    InvalidSelfHashRef, // 16

    MissingOwnerRef, // 17
    InvalidOwnerRef, // 18

    ThisContractIsNotTheOwnerOfThisToken, // 19

    OnlyOwnerCanCallThisFunction, // 20

    FailedToGetCallStack, // 21

    FailedToParseContractHash, // 22
}

impl From<StorageError> for ApiError {
    fn from(e: StorageError) -> Self {
        ApiError::User(e as u16)
    }
}
