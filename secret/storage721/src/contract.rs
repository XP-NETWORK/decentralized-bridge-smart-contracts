use cosmwasm_std::{
    entry_point, to_binary, Addr, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use secret_toolkit::utils::HandleCallback;

use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::snip721_msg::Snip721ExecuteMsg;
use crate::state::{COLLECTION721_ADDRESS, COLLECTION721_CODE, OWNER};
use crate::storage_deployer_msg::StorageDeployerInfo;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api
        .debug(format!("Storage 721 was initialized by {}", info.sender).as_str());

    OWNER.save(deps.storage, &msg.owner)?;
    COLLECTION721_CODE.save(deps.storage, &msg.collection_code_info)?;
    COLLECTION721_ADDRESS.save(deps.storage, &msg.collection_address)?;

    let offspring_info = StorageDeployerInfo {
        label: msg.collection_address.clone().into_string(),
        address: _env.contract.address,
        code_hash: _env.contract.code_hash,
        is_original: msg.is_original,
        token_id: msg.token_id,
        token_amount: 1,
        collection_code_hash: msg.collection_code_info.code_hash,
    };
    Ok(Response::new().set_data(to_binary(&offspring_info)?))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    if OWNER.load(deps.storage)? != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    match msg {
        // ExecuteMsg::DepositToken { token_id } => deposit_token(deps, env, token_id),
        ExecuteMsg::UnLockToken { token_id, to } => unlock_token(deps, token_id, to),
    }
}

// fn deposit_token(deps: DepsMut, env: Env, token_id: String) -> StdResult<Response> {
//     let collection_info = COLLECTION721_CODE.load(deps.storage)?;
//     let colection_address = COLLECTION721_ADDRESS.load(deps.storage)?;
//     let transfer_msg = Snip721ExecuteMsg::TransferNft {
//         recipient: env.contract.address.to_string(),
//         token_id: token_id.to_string(),
//         memo: Option::None,
//         padding: Option::None,
//     }
//     .to_cosmos_msg(
//         collection_info.code_hash,
//         colection_address.into_string(),
//         None,
//     )?;

//     Ok(Response::new().add_message(transfer_msg))
// }

fn unlock_token(deps: DepsMut, token_id: String, to: Addr) -> StdResult<Response> {
    let collection_info = COLLECTION721_CODE.load(deps.storage)?;
    let colection_address = COLLECTION721_ADDRESS.load(deps.storage)?;
    let transfer_msg = Snip721ExecuteMsg::TransferNft {
        recipient: to.to_string(),
        token_id: token_id.to_string(),
        memo: Option::None,
        padding: Option::None,
    }
    .to_cosmos_msg(
        collection_info.code_hash,
        colection_address.into_string(),
        None,
    )?;

    Ok(Response::new().add_message(transfer_msg))
}
