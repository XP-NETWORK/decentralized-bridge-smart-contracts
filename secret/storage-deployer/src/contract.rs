use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, DepsMut, Env, MessageInfo, Reply, Response, SubMsg,
    SubMsgResult,
};

use secret_toolkit::utils::{pad_handle_result, InitCallback};

use crate::bridge_msg::BridgeInfo;
use crate::error::ContractError;
use crate::state::{
    BLOCK_SIZE, STORAGE1155_CODE, STORAGE1155_INSTANTIATE_REPLY_ID, STORAGE721_CODE,
    STORAGE721_INSTANTIATE_REPLY_ID,
};
use crate::structs::ReplyStorageInfo;
use crate::{
    msg::{StorageDeployerExecuteMsg, StorageDeployerInstantiateMsg},
    state::OWNER,
    structs::CodeInfo,
};

use crate::offspring_msg::StorageInstantiateMsg;

////////////////////////////////////// Init ///////////////////////////////////////
/// Returns Result<Response, ContractError>
///
/// Initializes the offspring contract state.
///
/// # Arguments
///
/// * `deps`  - DepsMut containing all the contract's external dependencies
/// * `_env`  - Env of contract's environment
/// * `info`  - Carries the info of who sent the message and how much native funds were sent
/// * `msg`   - InitMsg passed in with the instantiation message
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: StorageDeployerInstantiateMsg,
) -> Result<Response, ContractError> {
    OWNER.save(deps.storage, &info.sender)?;
    STORAGE721_CODE.save(deps.storage, &msg.storage721_code_info)?;
    STORAGE1155_CODE.save(deps.storage, &msg.storage1155_code_info)?;

    let offspring_info = BridgeInfo {
        address: _env.contract.address,
    };
    Ok(Response::new().set_data(to_binary(&offspring_info)?))
}

///////////////////////////////////// Execute //////////////////////////////////////
/// Returns Result<Response, ContractError>
///
/// # Arguments
///
/// * `deps` - DepsMut containing all the contract's external dependencies
/// * `env`  - Env of contract's environment
/// * `info` - Carries the info of who sent the message and how much native funds were sent along
/// * `msg`  - HandleMsg passed in with the execute message
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: StorageDeployerExecuteMsg,
) -> Result<Response, ContractError> {
    deps.api
        .debug(format!(" in storage deployer execute").as_str());

    if OWNER.load(deps.storage)? != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let response = match msg {
        StorageDeployerExecuteMsg::CreateStorage721 {
            label,
            collection_address,
            collection_code_info,
            owner,
            is_original,
            token_id,
        } => try_create_storage_721(
            deps,
            env,
            label,
            collection_address,
            collection_code_info,
            owner,
            is_original,
            token_id,
        ),
        StorageDeployerExecuteMsg::CreateStorage1155 {
            label,
            collection_address,
            collection_code_info,
            owner,
            is_original,
            token_id,
        } => try_create_storage_1155(
            deps,
            env,
            label,
            collection_address,
            collection_code_info,
            owner,
            is_original,
            token_id,
        ),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

/// Returns Result<Response, ContractError>
///
/// create a new offspring
///
/// # Arguments
///
/// * `deps`        - DepsMut containing all the contract's external dependencies
/// * `env`         - Env of contract's environment
/// * `password`    - String containing the password to give the offspring
/// * `owner`       - address of the owner associated to this offspring contract
/// * `count`       - the count for the counter template
/// * `description` - optional free-form text string owner may have used to describe the offspring
fn try_create_storage_721(
    deps: DepsMut,
    _env: Env,
    label: String,
    collection_address: Addr,
    collection_code_info: CodeInfo,
    owner: String,
    is_original: bool,
    token_id: String,
) -> Result<Response, ContractError> {
    deps.api.debug(
        format!(
            " in storage deployer creating storage 721 {}",
            is_original
        )
        .as_str(),
    );

    let owner_addr = deps.api.addr_validate(&owner)?;

    let initmsg = StorageInstantiateMsg {
        collection_address,
        owner: owner_addr,
        collection_code_info,
        is_original,
        token_id,
    };

    let offspring_code = STORAGE721_CODE.load(deps.storage)?;
    let init_submsg = SubMsg::reply_always(
        initmsg.to_cosmos_msg(
            None,
            label,
            offspring_code.code_id,
            offspring_code.code_hash,
            None,
        )?,
        STORAGE721_INSTANTIATE_REPLY_ID,
    );
    // You can instead turn initmsg into a Cosmos message doing the following:
    /*
    let init_cosmos_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: offspring_code.code_id,
        code_hash: offspring_code.code_hash,
        msg: to_binary(&initmsg)?,
        funds: vec![],
        label,
    });
    */
    Ok(Response::new().add_submessage(init_submsg))
}

fn try_create_storage_1155(
    deps: DepsMut,
    _env: Env,
    label: String,
    collection_address: Addr,
    collection_code_info: CodeInfo,
    owner: String,
    is_original: bool,
    token_id: String,
) -> Result<Response, ContractError> {
    let owner_addr = deps.api.addr_validate(&owner)?;

    let initmsg = StorageInstantiateMsg {
        collection_address,
        owner: owner_addr,
        collection_code_info,
        is_original,
        token_id,
    };

    // pub collection_address: Addr,
    // pub owner: Addr,
    // pub collection_code_info: CodeInfo,
    // pub is_original: bool,
    // pub token_id: String,

    let offspring_code = STORAGE1155_CODE.load(deps.storage)?;
    let init_submsg = SubMsg::reply_always(
        initmsg.to_cosmos_msg(
            None,
            label,
            offspring_code.code_id,
            offspring_code.code_hash,
            None,
        )?,
        STORAGE1155_INSTANTIATE_REPLY_ID,
    );
    // You can instead turn initmsg into a Cosmos message doing the following:
    /*
    let init_cosmos_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: offspring_code.code_id,
        code_hash: offspring_code.code_hash,
        msg: to_binary(&initmsg)?,
        funds: vec![],
        label,
    });
    */
    Ok(Response::new().add_submessage(init_submsg))
}

/// Returns Result<Response, ContractError>
///
/// allows admin to edit the offspring contract version.
///
/// # Arguments
///
/// * `deps`                - DepsMut containing all the contract's external dependencies
/// * `info`                - Carries the info of who sent the message and how much native funds were sent along
/// * `offspring_code_info` - CodeInfo of the new offspring version
// fn try_new_storage_721(
//     deps: DepsMut,
//     info: MessageInfo,
//     storage_code_info: CodeInfo,
// ) -> Result<Response, ContractError> {
//     // only allow admin to do this
//     let sender = info.sender;
//     if OWNER.load(deps.storage)? != sender {
//         return Err(ContractError::Unauthorized {});
//     }
//     STORAGE721_CODE.save(deps.storage, &storage_code_info)?;

//     let resp_data = to_binary(&HandleAnswer::Status {
//         status: ResponseStatus::Success,
//         message: None,
//     })?;
//     Ok(Response::new().set_data(resp_data))
// }

// fn try_new_storage_1155(
//     deps: DepsMut,
//     info: MessageInfo,
//     storage_code_info: CodeInfo,
// ) -> Result<Response, ContractError> {
//     // only allow admin to do this
//     let sender = info.sender;
//     if OWNER.load(deps.storage)? != sender {
//         return Err(ContractError::Unauthorized {});
//     }
//     STORAGE1155_CODE.save(deps.storage, &storage_code_info)?;

//     let resp_data = to_binary(&HandleAnswer::Status {
//         status: ResponseStatus::Success,
//         message: None,
//     })?;
//     Ok(Response::new().set_data(resp_data))
// }

/////////////////////////////////////// Reply /////////////////////////////////////
/// Returns Result<Response, ContractError>
///
/// # Arguments
///
/// * `deps` - DepsMut containing all the contract's external dependencies
/// * `msg` - QueryMsg passed in with the query call
#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        STORAGE721_INSTANTIATE_REPLY_ID => handle_instantiate_reply_721(msg),
        STORAGE1155_INSTANTIATE_REPLY_ID => handle_instantiate_reply_1155(msg),
        id => Err(ContractError::UnexpectedReplyId { id }),
    }
}

fn handle_instantiate_reply_721(msg: Reply) -> Result<Response, ContractError> {
    // The parsing process below can be handled easier if one imports cw-plus
    // See: https://github.com/CosmWasm/cw-plus/blob/main/packages/utils/src/parse_reply.rs
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyStorageInfo = from_binary(&bin)?;
                register_storage_721_impl(reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with contract address 721".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

fn handle_instantiate_reply_1155(msg: Reply) -> Result<Response, ContractError> {
    // The parsing process below can be handled easier if one imports cw-plus
    // See: https://github.com/CosmWasm/cw-plus/blob/main/packages/utils/src/parse_reply.rs
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyStorageInfo = from_binary(&bin)?;
                register_storage_1155_impl(reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with contract address 1155".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

/// Returns Result<Response, ContractError>
///
/// Registers the calling offspring by saving its info and adding it to the appropriate lists
///
/// # Arguments
///
/// * `deps`       - DepsMut containing all the contract's external dependencies
/// * `reply_info` - reference to ReplyOffspringInfo of the offspring that is trying to register
fn register_storage_721_impl(reply_info: ReplyStorageInfo) -> Result<Response, ContractError> {
    // Ok(Response::new().set_data(to_binary(&reply_info)?))
    Ok(Response::new()
        .add_attribute("storage_address_721", &reply_info.address)
        .set_data(to_binary(&reply_info)?))
}

fn register_storage_1155_impl(reply_info: ReplyStorageInfo) -> Result<Response, ContractError> {
    // Ok(Response::new().set_data(to_binary(&reply_info)?))
    Ok(Response::new()
        .add_attribute("storage_address_1155", &reply_info.address)
        .set_data(to_binary(&reply_info)?))
}
