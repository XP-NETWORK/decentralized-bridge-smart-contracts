use cosmwasm_std::Addr;
use serde::Serialize;

use secret_toolkit::utils::HandleCallback;

use crate::state::BLOCK_SIZE;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Storage721ExecuteMsg {
    UnLockToken { token_id: String, to: Addr },
}

impl HandleCallback for Storage721ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
