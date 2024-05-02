use self::contract::methods;
use self::error::ContractError;
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw721_base::Cw721Contract;
use init::InstantiateMsg;
use msg::CW2981QueryMsg;
use royalty::RoyaltyData;

pub mod constants;
pub mod contract;
pub mod error;
pub mod init;
pub mod msg;
pub mod query;
pub mod reply;
pub mod royalty;
#[cfg(test)]
pub mod tests;

pub type MintExtension = RoyaltyData;
pub type NftContract<'a> = Cw721Contract<'a, MintExtension, Empty, Empty, CW2981QueryMsg>;
pub type NftExecuteMsg = cw721_base::ExecuteMsg<MintExtension, Empty>;
pub type NftQueryMsg = cw721_base::QueryMsg<CW2981QueryMsg>;

#[cfg(not(feature = "library"))]
pub mod entry {

    use super::*;

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        methods::init(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: NftExecuteMsg,
    ) -> Result<Response, ContractError> {
        methods::execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: NftQueryMsg) -> StdResult<Binary> {
        methods::query(deps, env, msg)
    }
}
