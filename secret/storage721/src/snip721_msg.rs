use serde::Serialize;

use secret_toolkit::{snip721::Expiration, utils::HandleCallback};

use crate::state::BLOCK_SIZE;

/// Factory handle messages to be used by offspring.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Snip721ExecuteMsg {
    Approve {
        /// address being granted the permission
        spender: String,
        /// id of the token that the spender can transfer
        token_id: String,
        /// optional expiration for this approval
        expires: Option<Expiration>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// transfer a token if it is transferable
    TransferNft {
        /// recipient of the transfer
        recipient: String,
        /// id of the token to transfer
        token_id: String,
        /// optional memo for the tx
        memo: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
}

impl HandleCallback for Snip721ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
