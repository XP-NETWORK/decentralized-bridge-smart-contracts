use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum BridgeError {
    AlreadyInitialized, // 0
    InvalidDestinationChain, // 1

    MissingArgumentNewValidator,  // 2
    InvalidArgumentNewValidator,  // 3

    MissingArgumentSignatures,  // 4
    InvalidArgumentSignatures,  // 5

    MissingValidatorsDictRef, // 6
    InvalidValidatorsDictRef, // 7

    MissingChainTypeRef, // 8
    InvalidChainTypeRef, // 9

    FailedToPrepareSignature, // 10
    FailedToPreparePublicKey, // 11
    FailedToCreateDictionary, // 12

    MissingValidatorsCountRef, // 13
    InvalidValidatorsCountRef, // 14

    ThresholdNotReached, // 15
    FailedToGetArgBytes, // 16
    UnexpectedKeyVariant, // 17
    FeeLessThanSentAmount, //18

    MissingArgumentTokenId,  // 19
    InvalidArgumentTokenId,  // 20

    MissingArgumentDestinationChain,  // 21
    InvalidArgumentDestinationChain,  // 22

    MissingArgumentDestinationUserAddress, // 23
    InvalidArgumentDestinationUserAddress, // 24

    MissingArgumentSourceNftContractAddress, // 25
    InvalidArgumentSourceNftContractAddress, // 26

    MissingArgumentMetadataUri, // 27
    InvalidArgumentMetadataUri, // 28
}

impl From<BridgeError> for ApiError {
    fn from(e: BridgeError) -> Self {
        ApiError::User(e as u16)
    }
}
