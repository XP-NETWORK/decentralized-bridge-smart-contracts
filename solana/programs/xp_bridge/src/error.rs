use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Tx Fees is required!")]
    TxFeeReq,

    #[msg("Failed to verify signature!")]
    InvalidSignature,

    #[msg("Invalid destination chain!")]
    InvalidDestination,

    #[msg("Fee and sent amount do not match!")]
    FeeNotMatch,

    #[msg("Must have signatures!")]
    NoSignatures,

    #[msg("Threshold not reached!")]
    NoThreshold,

    #[msg("Validator does not exist!")]
    NotValidator,

    #[msg("validator already exist")]
    ValidatorAleadyExist,

    #[msg("Invalid NFT type!")]
    InvalidNft,

    #[msg("Invalid Token Amount!")]
    InvalidTokenAmount,

    #[msg("Data already processed!")]
    DataProcessed,

    #[msg("instruction at wrong index")]
    InstructionAtWrongIndex,
    #[msg("invalid ed25519 instruction")]
    InvalidEd25519Instruction,
    #[msg("invalid group key")]
    InvalidGroupKey,
    #[msg("invalid program id")]
    InvalidProgramId,
    #[msg("invalid args")]
    InvalidArgs,

    #[msg("Signature verification failed.")]
    SigVerificationFailed,
}
