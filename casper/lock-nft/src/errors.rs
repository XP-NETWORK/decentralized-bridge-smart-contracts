use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum LockError {
    FailedToPrepareServicePublicKey, // 0
    FailedToTransferFunds, // 1
    FailedToDecodeHex, // 2
}

impl From<LockError> for ApiError {
    fn from(e: LockError) -> Self {
        ApiError::User(e as u16)
    }
}
