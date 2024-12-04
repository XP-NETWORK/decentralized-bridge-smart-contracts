use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ClaimError {
    FailedToPrepareServicePublicKey, // 0
    FailedToTransferFunds,           // 1
    FailedToDecodeHex,               // 2
    FeeLessThanSentAmount,           // 3
}

impl From<ClaimError> for ApiError {
    fn from(e: ClaimError) -> Self {
        ApiError::User(e as u16)
    }
}
