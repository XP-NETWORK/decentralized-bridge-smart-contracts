use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum StorageError {
    AlreadyInitialized, // 0

    MissingArgumentCollection,  // 1
    InvalidArgumentCollection,  // 2

    MissingArgumentOwner, // 3
    InvalidArgumentOwner, // 4

    MissingArgumentTokenId, // 5
    InvalidArgumentTokenId, // 6

    MissingArgumentTo, // 7
    InvalidArgumentTo, // 8

    FailedToGetArgBytes, // 9
    UnexpectedKeyVariant, // 10
}

impl From<StorageError> for ApiError {
    fn from(e: StorageError) -> Self {
        ApiError::User(e as u16)
    }
}
