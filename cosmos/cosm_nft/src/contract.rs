use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    constants::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
    init::InstantiateMsg,
    msg::CW2981QueryMsg,
    query::{check_royalties, query_royalties_info},
    reply::ReplyCollectionInfo,
    royalty::RoyaltyData,
    NftContract, NftExecuteMsg, NftQueryMsg,
};

pub mod methods {

    use super::*;

    pub fn init(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let response = to_json_binary(&ReplyCollectionInfo {
            label: msg.name.clone(),
            owner: Addr::unchecked(msg.minter.clone()),
            address: env.contract.address.clone(),
            destination_user_address: msg.destination_user_address,
            royalty_receiver: msg.royalty_receiver,
            metadata: msg.metadata,
            source_nft_contract_address: msg.source_nft_contract_address,
            source_chain: msg.source_chain,
            token_id: msg.token_id,
            token_amount: msg.token_amount,
            royalty: msg.royalty,
            transaction_hash: msg.transaction_hash,
            lock_tx_chain: msg.lock_tx_chain
        })?;
        Ok(NftContract::default()
            .instantiate(
                deps.branch(),
                env,
                info,
                cw721_base::InstantiateMsg {
                    minter: msg.minter,
                    name: msg.name,
                    symbol: msg.symbol,
                },
            )
            .map(|r| r.set_data(response))?)
    }

    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: NftExecuteMsg,
    ) -> Result<Response, ContractError> {
        if let NftExecuteMsg::Mint {
            extension: RoyaltyData {
                royalty_percentage, ..
            },
            ..
        } = &msg
        {
            // validate royalty_percentage to be between 0 and 100
            // no need to check < 0 because royalty_percentage is u64
            if *royalty_percentage > 100 {
                return Err(ContractError::InvalidRoyaltyPercentage);
            }
        }

        NftContract::default()
            .execute(deps, env, info, msg)
            .map_err(Into::into)
    }

    pub fn query(deps: Deps, env: Env, msg: NftQueryMsg) -> StdResult<Binary> {
        match msg {
            NftQueryMsg::Extension { msg } => match msg {
                CW2981QueryMsg::RoyaltyInfo {
                    token_id,
                    sale_price,
                } => to_json_binary(&query_royalties_info(deps, token_id, sale_price)?),
                CW2981QueryMsg::CheckRoyalties {} => to_json_binary(&check_royalties(deps)?),
            },
            _ => NftContract::default().query(deps, env, msg),
        }
    }
}
