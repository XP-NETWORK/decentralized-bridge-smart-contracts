use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum BridgeError {
    AlreadyInitialized, // 0
    MissingArgumentGroupKey,  // 1
    InvalidArgumentGroupKey,  // 2
    MissingArgumentFeePublicKey,  // 3
    InvalidArgumentFeePublicKey,  // 4
    MissingArgumentSigData, // 5
    InvalidArgumentSigData, // 6
    MissingArgumentActionID, // 7
    InvalidArgumentActionID, // 8
    MissingArgumentMintWith, // 9
    InvalidArgumentMintWith, // 10
    MissingArgumentReceiver, // 11
    InvalidArgumentReceiver, // 12
    MissingArgumentMetadata, // 13
    InvalidArgumentMetadata, // 14
    MissingArgumentTokenID, // 15
    InvalidArgumentTokenID, // 16
    MissingArgumentChainNonce, // 17
    InvalidArgumentChainNonce, // 18
    MissingArgumentAmount, // 19
    InvalidArgumentAmount, // 20
    MissingArgumentTo, // 21
    InvalidArgumentTo, // 22
    MissingArgumentContract, // 23
    InvalidArgumentContract, // 24
    MissingConsumedActionsUref, // 25
    InvalidConsumedActionsUref, // 26
    MissingGroupKeyUref, // 27
    InvalidGroupKeyUref, // 28
    RetryingConsumedActions, // 29
    UnauthorizedAction, // 30
    ContractStatePaused, // 31
    FailedToTransferBwPursees, // 32
    IncorrectFeeSig, // 33
    MissingThisContractUref, // 34
    InvalidThisContractUref, // 35
    MissingFeePublicKeyUref, // 36
    InvalidFeePublicKeyUref, // 37
    MissingThisPurseUref, // 38
    InvalidThisPurseUref, // 39
    NotWhitelistedContract, // 40
    UnexpectedKeyVariant, // 41
    FailedToCreateDictionary, // 42
    FailedToGetArgBytes, // 43
    FailedToSerializeTxFee, // 44
    FailedToSerializeActionStruct, // 45
    FailedToPrepareSignature, // 46
    FailedToGetDictItem, // 47
    FailedToPreparePublicKey, // 48
    ThresholdNotReached, // 49
    FailedToGetValidator, // 50
    lol, // 51,
    lol2, // 52
    lol3, // 53
}

impl From<BridgeError> for ApiError {
    fn from(e: BridgeError) -> Self {
        ApiError::User(e as u16)
    }
}
