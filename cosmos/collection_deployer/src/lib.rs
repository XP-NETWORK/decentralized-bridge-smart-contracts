use cosmwasm_std::{from_json, to_json_binary, DepsMut, Reply, Response};

use error::CollectionFactoryContractError;
use msg::ReplyCollectionInfo;

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
        entry_point, to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env,
        MessageInfo, Reply, Response, SubMsg,
    };

    use crate::{
        bridge_msg::BridgeInfo,
        error::CollectionFactoryContractError,
        handle_instantiate_reply_721,
        msg::{CollectionDeployerExecuteMsg, CollectionDeployerInstantiateMsg},
        state::{OWNER, SNIP721_CODE, SNIP721_INSTANTIATE_REPLY_ID},
    };

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: CollectionDeployerInstantiateMsg,
    ) -> Result<Response, CollectionFactoryContractError> {
        OWNER.save(deps.storage, &info.sender)?;
        SNIP721_CODE.save(deps.storage, &msg.collection721_code_id)?;

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
        msg: CollectionDeployerExecuteMsg,
    ) -> Result<Response, CollectionFactoryContractError> {
        if OWNER.load(deps.storage)? != info.sender {
            return Err(CollectionFactoryContractError::Unauthorized);
        }

        let response = match msg {
            CollectionDeployerExecuteMsg::CreateCollection721 {
                owner,
                name,
                symbol,
                source_nft_contract_address,
                source_chain,
                destination_user_address,
                token_id,
                token_amount,
                royalty,
                royalty_receiver,
                metadata,
                transaction_hash,
            } => try_create_collection_721(
                deps,
                env,
                owner,
                name,
                symbol,
                source_nft_contract_address,
                source_chain,
                destination_user_address,
                token_id,
                token_amount,
                royalty,
                royalty_receiver,
                metadata,
                transaction_hash,
            ),
        }?;

        Ok(response)
    }

    fn try_create_collection_721(
        deps: DepsMut,
        _env: Env,
        owner: String,
        name: String,
        symbol: String,
        source_nft_contract_address: String,
        source_chain: String,
        destination_user_address: Addr,
        token_id: String,
        token_amount: u128,
        royalty: u16,
        royalty_receiver: Addr,
        metadata: String,
        transaction_hash: String,
    ) -> Result<Response, CollectionFactoryContractError> {
        let owner_addr = deps.api.addr_validate(&owner)?;

        let initmsg = cosm_nft::init::InstantiateMsg {
            name: name.clone(),
            symbol: symbol.clone(),
            minter: owner_addr.to_string(),
            source_nft_contract_address,
            source_chain,
            destination_user_address,
            token_id,
            token_amount,
            royalty,
            royalty_receiver,
            metadata,
            transaction_hash,
        };

        let code_id = SNIP721_CODE.load(deps.storage)?;

        let init_submsg = SubMsg::reply_always(
            CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
                admin: None,
                code_id,
                msg: to_json_binary(&initmsg)?,
                funds: vec![],
                label: name,
            }),
            SNIP721_INSTANTIATE_REPLY_ID,
        );
        Ok(Response::new().add_submessage(init_submsg))
    }

    #[entry_point]
    pub fn reply(
        _deps: DepsMut,
        _env: Env,
        msg: Reply,
    ) -> Result<Response, CollectionFactoryContractError> {
        match msg.id {
            SNIP721_INSTANTIATE_REPLY_ID => handle_instantiate_reply_721(msg, _deps),
            id => Err(CollectionFactoryContractError::UnexpectedReplyId { id }),
        }
    }

    #[entry_point]
    pub fn query(
        _deps: Deps,
        _env: Env,
        _msg: Empty,
    ) -> Result<Binary, CollectionFactoryContractError> {
        Ok(Binary::default())
    }
}

fn handle_instantiate_reply_721(
    msg: Reply,
    _deps: DepsMut,
) -> Result<Response, CollectionFactoryContractError> {
    let result = cw0::parse_reply_instantiate_data(msg.clone()).map_err(|e| {
        CollectionFactoryContractError::CustomError(format!(
            "Failed to parse instantiate reply: {}",
            e
        ))
    })?;
    if let Some(bin) = result.data {
        let reply_info: ReplyCollectionInfo = from_json(&bin)?;
        register_collection_721_impl(reply_info)
    } else {
        Err(CollectionFactoryContractError::CustomError(
            "Init didn't response with contract address 721".to_string(),
        ))
    }
}

fn register_collection_721_impl(
    reply_info: ReplyCollectionInfo,
) -> Result<Response, CollectionFactoryContractError> {
    Ok(Response::new()
        .add_attribute("collection_address_721", &reply_info.address)
        .set_data(to_json_binary(&reply_info)?))
}
