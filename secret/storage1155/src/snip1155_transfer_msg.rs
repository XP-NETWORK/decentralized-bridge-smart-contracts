use cosmwasm_std::{Addr, Uint128};
use serde::Serialize;

use secret_toolkit::{snip721::Expiration, utils::HandleCallback};

use crate::state::BLOCK_SIZE;

/// Factory handle messages to be used by offspring.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Snip1155ExecuteMsg {
    GivePermission {
        /// address being granted/revoked permission
        allowed_address: Addr,
        /// token id to apply approval/revocation to.
        /// Additional Spec feature: if == None, perform action for all owner's `token_id`s
        token_id: String,
        /// optional permission level for viewing balance. If ignored, leaves current permission settings
        view_balance: Option<bool>,
        view_balance_expiry: Option<Expiration>,
        /// optional permission level for viewing private metadata. If ignored, leaves current permission settings
        view_private_metadata: Option<bool>,
        view_private_metadata_expiry: Option<Expiration>,
        /// set allowance by for transfer approvals. If ignored, leaves current permission settings
        transfer: Option<Uint128>,
        transfer_expiry: Option<Expiration>,
        /// optional message length padding
        padding: Option<String>,
    },
    Transfer {
        token_id: String,
        // equivalent to `owner` in SNIP20. Tokens are sent from this address.
        from: Addr,
        recipient: Addr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
}

impl HandleCallback for Snip1155ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
