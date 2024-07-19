use cosmwasm_std::{from_json, to_json_binary, Reply, Response};
use error::StorageFactoryContractError;
use msg::ReplyStorageInfo;

mod bridge_msg;
pub mod error;
pub mod msg;
mod state;
mod storage_deployer_msg;
mod storage_info;
#[cfg(test)]
mod tests;

#[cfg(not(feature = "library"))]
pub mod entry {

    use cosmwasm_std::{
        entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
        Reply, Response, SubMsg,
    };

    use crate::{
        bridge_msg::BridgeInfo,
        error::StorageFactoryContractError,
        handle_instantiate_reply_721,
        msg::StoreFactoryInstantiateMsg,
        state::{OWNER, STORAGE721_CODE, STORAGE721_INSTANTIATE_REPLY_ID},
        storage_info::StorageInstantiateMsg,
    };

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: StoreFactoryInstantiateMsg,
    ) -> Result<Response, StorageFactoryContractError> {
        OWNER.save(deps.storage, &info.sender)?;
        STORAGE721_CODE.save(deps.storage, &msg.storage721_code_id)?;

        let offspring_info = BridgeInfo {
            address: env.contract.address,
        };
        Ok(Response::new().set_data(to_json_binary(&offspring_info)?))
    }
    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: crate::msg::StoreFactoryExecuteMsg,
    ) -> Result<Response, StorageFactoryContractError> {
        if OWNER.load(deps.storage)? != info.sender {
            return Err(StorageFactoryContractError::Unauthorized);
        }

        match msg {
            crate::msg::StoreFactoryExecuteMsg::CreateStorage721 {
                label,
                collection_address,
                collection_code_id,
                owner,
                is_original,
                token_id,
            } => try_create_storage_721(
                deps,
                env,
                label,
                collection_address,
                collection_code_id,
                owner,
                is_original,
                token_id,
            ),
        }
    }

    fn try_create_storage_721(
        deps: DepsMut<'_>,
        _env: Env,
        label: String,
        collection_address: cosmwasm_std::Addr,
        collection_code_id: u64,
        owner: String,
        is_original: bool,
        token_id: String,
    ) -> Result<Response, StorageFactoryContractError> {
        let owner_addr = deps.api.addr_validate(&owner)?;

        let initmsg = StorageInstantiateMsg {
            collection_address,
            owner: owner_addr,
            collection_code_id,
            is_original,
            token_id,
        };

        let offspring_code_id = STORAGE721_CODE.load(deps.storage)?;

        let init_submsg = SubMsg::reply_always(
            CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
                admin: None,
                code_id: offspring_code_id,
                msg: to_json_binary(&initmsg)?,
                funds: vec![],
                label,
            }),
            STORAGE721_INSTANTIATE_REPLY_ID,
        );

        Ok(Response::new().add_submessage(init_submsg))
    }

    #[entry_point]
    pub fn query(
        _deps: Deps,
        _env: Env,
        _msg: Empty,
    ) -> Result<Binary, StorageFactoryContractError> {
        Ok(Binary::default())
    }

    #[entry_point]
    pub fn reply(
        _deps: DepsMut,
        _env: Env,
        msg: Reply,
    ) -> Result<Response, StorageFactoryContractError> {
        match msg.id {
            STORAGE721_INSTANTIATE_REPLY_ID => handle_instantiate_reply_721(msg),
            id => Err(StorageFactoryContractError::UnexpectedReplyId { id }),
        }
    }
}

fn handle_instantiate_reply_721(msg: Reply) -> Result<Response, StorageFactoryContractError> {
    let reply = cw0::parse_reply_instantiate_data(msg).map_err(|e| {
        StorageFactoryContractError::CustomError(format!(
            "Failed to parse instantiate reply: {}",
            e
        ))
    })?;
    if let Some(bin) = reply.data {
        let reply_info: ReplyStorageInfo = from_json(&bin)?;
        register_storage_721_impl(reply_info)
    } else {
        Err(StorageFactoryContractError::CustomError(
            "Init didn't response with store address 721".to_string(),
        ))
    }
}

fn register_storage_721_impl(
    reply_info: ReplyStorageInfo,
) -> Result<Response, StorageFactoryContractError> {
    // Ok(Response::new().set_data(to_binary(&reply_info)?))
    Ok(Response::new()
        .add_attribute("storage_address_721", &reply_info.address)
        .set_data(to_json_binary(&reply_info)?))
}
