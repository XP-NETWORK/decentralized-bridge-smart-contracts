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

    MissingArgumentMetadata, // 27
    InvalidArgumentMetadata, // 28

    MissingOTDDictRef, // 29
    InvalidOTDDictRef, // 30

    MissingOSRef, // 31
    InvalidOSRef, // 32

    MissingDSRef, // 33
    InvalidDSRef, // 34

    MissingSelfHashRef, // 35
    InvalidSelfHashRef, // 36

    CouldntGetPurseBalance, // 37

    MissingArgumentCallerPurse, // 38
    InvalidArgumentCallerPurse, // 39

    MissingServiceAddressRef, // 40
    InvalidServiceAddressRef, // 41

    MissingArgumentStorageAddress, // 42
    InvalidArgumentStorageAddress, // 43

    InvalidServiceAddress, // 44

    FailedToUnWrapStorageAddress, // 45

    InvalidStorageAddress, // 46

    MissingArgumentNftSenderAddress, // 47
    InvalidArgumentNftSenderAddress, // 48

    FailedToRegisterOwnerToCollection, // 49

    MissingArgumentSourceChain,  // 50
    InvalidArgumentSourceChain,  // 51

    MissingArgumentName, // 52
    InvalidArgumentName, // 53

    MissingArgumentSymbol, // 54
    InvalidArgumentSymbol, // 55

    MissingArgumentRoyalty, // 56
    InvalidArgumentRoyalty, // 57

    MissingArgumentRoyaltyReceiver, // 58
    InvalidArgumentRoyaltyReceiver, // 59

    MissingArgumentTransactionHash, // 60
    InvalidArgumentTransactionHash, // 61

    MissingArgumentTokenAmount, // 62
    InvalidArgumentTokenAmount, // 63

    MissingArgumentNftType, // 64
    InvalidArgumentNftType, // 65

    MissingArgumentFee, // 66
    InvalidArgumentFee, // 67

    MissingArgumentLockTxChain, // 68
    InvalidArgumentLockTxChain, // 69

    MissingArgumentCollectionAddress, // 70
    InvalidArgumentCollectionAddress, // 71

    InvalidNftType, // 72

    MissingUniqueIdentifiersDictRef, // 73
    InvalidUniqueIdentifiersDictRef, // 74

    DataAlreadyProcessed, // 75

    MissingDTODictRef, // 76
    InvalidDTODictRef, // 77

    InvalidFee, // 78

    MissingThisPurseRef, // 79
    InvalidThisPurseRef, // 80

    NoRewardsAvailable, // 81

    FailedToGetValidatorForReward, // 82

    FailedToUnlockNft, // 83

    FailedToMintNft, // 84

    InvalidBridgeState, // 85

    CantBeThere, // 86

    MissingArgumentValidator,  // 87
    InvalidArgumentValidator,  // 88

    ValidatorNotAdded, // 89

    MissingBlackListValidatorsDictRef, // 90
    InvalidBlackListValidatorsDictRef, // 91

    AlreadyBlacklisted, // 92

    ValidatorDoesNotExist, // 93

    MissingArgumentSenderPurse, // 94
    InvalidArgumentSenderPurse, // 95

    MissingArgumentAmount, // 96
    InvalidArgumentAmount, // 97

    MissingStorageDeployFeeRef, // 98
    InvalidStorageDeployFeeRef, // 99

    MissingCollectionDeployFeeRef, // 100
    InvalidCollectionDeployFeeRef, // 101

    MissingClaimFeeRef, // 102
    InvalidClaimFeeRef, // 103

    StorageFeeLessThanSentAmount, // 104

    CollectionFeeLessThanSentAmount, // 105
}

impl From<BridgeError> for ApiError {
    fn from(e: BridgeError) -> Self {
        ApiError::User(e as u16)
    }
}
