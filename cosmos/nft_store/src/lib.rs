use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, DepsMut, Empty, Response};
use error::StorageContractError;
use state::COLLECTION721_ADDRESS;

pub mod error;
pub mod msg;
mod state;
mod storage_deployer_msg;
#[cfg(test)]
mod tests;

#[cfg(not(feature = "library"))]
pub mod entry {

    use cosmwasm_std::{
        entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    };

    use crate::{
        error::StorageContractError,
        msg::{NftStoreExecuteMsg, NftStoreInstantiateMsg, NftStoreQueryMsg},
        state::{COLLECTION721_ADDRESS, OWNER},
        storage_deployer_msg::StorageDeployerInfo,
        unlock_token,
    };

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        msg: NftStoreInstantiateMsg,
    ) -> Result<Response, StorageContractError> {
        OWNER.save(deps.storage, &msg.owner)?;
        COLLECTION721_ADDRESS.save(deps.storage, &msg.collection_address)?;

        let offspring_info = StorageDeployerInfo {
            label: msg.collection_address.clone().into_string(),
            address: env.contract.address,
            is_original: msg.is_original,
            token_id: msg.token_id,
            token_amount: 1,
        };
        Ok(Response::new().set_data(to_json_binary(&offspring_info)?))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: NftStoreExecuteMsg,
    ) -> Result<Response, StorageContractError> {
        if OWNER.load(deps.storage)? != info.sender {
            return Err(StorageContractError::Unauthorized);
        }
        match msg {
            NftStoreExecuteMsg::UnLockToken { token_id, to } => unlock_token(deps, token_id, to),
        }
    }

    #[entry_point]
    pub fn query(
        deps: Deps,
        _env: Env,
        msg: NftStoreQueryMsg,
    ) -> Result<Binary, StorageContractError> {
        match msg {
            NftStoreQueryMsg::GetCollectionAddress => {
                Ok(to_json_binary(&COLLECTION721_ADDRESS.load(deps.storage)?)?)
            }
        }
    }
}

fn unlock_token(
    deps: DepsMut,
    token_id: String,
    to: Addr,
) -> Result<Response, StorageContractError> {
    let colection_address = COLLECTION721_ADDRESS.load(deps.storage)?;
    let transfer_msg = cw721_base::msg::ExecuteMsg::<Empty, Empty>::TransferNft {
        recipient: to.to_string(),
        token_id: token_id.to_string(),
    };
    let msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: colection_address.to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });
    Ok(Response::new().add_message(msg))
}
