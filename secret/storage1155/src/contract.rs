use cosmwasm_std::{
    entry_point, to_binary, Addr, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use secret_toolkit::utils::HandleCallback;

use crate::msg::{Storage1155ExecuteMsg, Storage1155InstantiateMsg};
use crate::snip1155_transfer_msg::Snip1155ExecuteMsg;
use crate::state::{COLLECTION1155_ADDRESS, COLLECTION1155_CODE, OWNER};
use crate::storage_deployer_msg::StorageDeployerInfo;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Storage1155InstantiateMsg,
) -> StdResult<Response> {
    deps.api
        .debug(format!("Storage 1155 was initialized by {}", info.sender).as_str());

    OWNER.save(deps.storage, &msg.owner)?;
    COLLECTION1155_CODE.save(deps.storage, &msg.collection_code_info)?;
    COLLECTION1155_ADDRESS.save(deps.storage, &msg.collection_address)?;

    let offspring_info = StorageDeployerInfo {
        label: msg.collection_address.clone().into_string() + &env.block.time.seconds().to_string(),
        address: env.contract.address,
        code_hash: env.contract.code_hash,
        is_original: msg.is_original,
        token_id: msg.token_id,
        token_amount: msg.token_amount,
        collection_code_hash: msg.collection_code_info.code_hash,
        from: msg.from,
        source_nft_contract_address: msg.source_nft_contract_address
    };
    Ok(Response::new().set_data(to_binary(&offspring_info)?))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: Storage1155ExecuteMsg) -> StdResult<Response> {
    if OWNER.load(deps.storage)? != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    match msg {
        // ExecuteMsg::DepositToken { token_id, amount } => {
        //     deposit_token(deps, env, _info, token_id, amount)
        // }
        Storage1155ExecuteMsg::UnLockToken {
            token_id,
            amount,
            to,
        } => unlock_token(deps, env, token_id, amount, to),
    }
}

// fn deposit_token(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     token_id: String,
//     amount: u128,
// ) -> StdResult<Response> {
//     let collection_info = COLLECTION1155_CODE.load(deps.storage)?;
//     let collection_address = COLLECTION721_ADDRESS.load(deps.storage)?;
//     let transfer_msg = Snip1155ExecuteMsg::Transfer {
//         token_id: token_id.to_string(),
//         from: info.sender,
//         recipient: env.contract.address,
//         amount: amount.into(),
//         memo: Option::None,
//         padding: Option::None,
//     }
//     .to_cosmos_msg(
//         collection_info.code_hash,
//         collection_address.into_string(),
//         None,
//     )?;

//     Ok(Response::new().add_message(transfer_msg))
// }

fn unlock_token(
    deps: DepsMut,
    env: Env,
    token_id: String,
    amount: u128,
    to: Addr,
) -> StdResult<Response> {
    let collection_info = COLLECTION1155_CODE.load(deps.storage)?;
    let collection_address = COLLECTION1155_ADDRESS.load(deps.storage)?;
    let transfer_msg = Snip1155ExecuteMsg::Transfer {
        token_id: token_id.to_string(),
        from: env.contract.address,
        recipient: to,
        amount: amount.into(),
        memo: Option::None,
        padding: Option::None,
    }
    .to_cosmos_msg(
        collection_info.code_hash,
        collection_address.into_string(),
        None,
    )?;

    Ok(Response::new().add_message(transfer_msg))
}
