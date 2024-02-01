use cosmwasm_std::Addr;
use serde::Serialize;

use secret_toolkit::utils::HandleCallback;

use crate::state::BLOCK_SIZE;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Storage1155ExecuteMsg {
    UnLockToken {
        token_id: String,
        amount: u128,
        to: Addr,
    },
}

impl HandleCallback for Storage1155ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
