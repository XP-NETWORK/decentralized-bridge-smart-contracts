use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, DepsMut, Env, MessageInfo, Reply, Response, SubMsg,
    SubMsgResult,
};

use secret_toolkit::utils::{pad_handle_result, InitCallback};
use snip1155::state::state_structs::CurateTokenId;

use crate::bridge_msg::BridgeInfo;
use crate::error::ContractError;
use crate::state::{
    BLOCK_SIZE, SNIP1155_CODE, SNIP1155_INSTANTIATE_REPLY_ID, SNIP721_CODE,
    SNIP721_INSTANTIATE_REPLY_ID,
};
use crate::structs::ReplyCollectionInfo;
use crate::{
    msg::{CollectionDeployerExecuteMsg, CollectionDeployerInstantiateMsg},
    state::OWNER,
};

use crate::offspring_msg::{
    Collection1155InstantiateMsg, Collection721InstantiateMsg,
};

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
    msg: CollectionDeployerInstantiateMsg,
) -> Result<Response, ContractError> {
    OWNER.save(deps.storage, &info.sender)?;
    SNIP721_CODE.save(deps.storage, &msg.collection721_code_info)?;
    SNIP1155_CODE.save(deps.storage, &msg.collection1155_code_info)?;

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
    msg: CollectionDeployerExecuteMsg,
) -> Result<Response, ContractError> {
    if OWNER.load(deps.storage)? != info.sender {
        return Err(ContractError::Unauthorized {});
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
            transaction_hash
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
            transaction_hash
        ),
        CollectionDeployerExecuteMsg::CreateCollection1155 {
            name,
            symbol,
            has_admin,
            admin,
            curators,
            initial_tokens,
            entropy,
            label,
            source_nft_contract_address,
            source_chain,
            destination_user_address,
            token_id,
            token_amount,
            royalty,
            royalty_receiver,
            metadata,
            transaction_hash
        } => try_create_collection_1155(
            deps,
            env,
            name,
            symbol,
            has_admin,
            admin,
            curators,
            initial_tokens,
            entropy,
            label,
            source_nft_contract_address,
            source_chain,
            destination_user_address,
            token_id,
            token_amount,
            royalty,
            royalty_receiver,
            metadata,
            transaction_hash
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
    transaction_hash:String
) -> Result<Response, ContractError> {
    let owner_addr = deps.api.addr_validate(&owner)?;

    // let factory = ContractInfo {
    //     code_hash: env.contract.code_hash,
    //     address: env.contract.address,
    // };

    let initmsg = Collection721InstantiateMsg {
        label: name.clone() + &symbol,
        owner: owner_addr.clone(),
        admin: Some(owner_addr.into_string()),
        name: name.clone(),
        symbol: symbol.clone(),
        entropy: name.clone() + &symbol,
        source_nft_contract_address,
        source_chain,
        destination_user_address,
        token_id,
        token_amount,
        royalty,
        royalty_receiver,
        metadata,
        transaction_hash
    };

    let offspring_code = SNIP721_CODE.load(deps.storage)?;
    let init_submsg = SubMsg::reply_always(
        initmsg.to_cosmos_msg(
            None,
            name + &symbol,
            offspring_code.code_id,
            offspring_code.code_hash,
            None,
        )?,
        SNIP721_INSTANTIATE_REPLY_ID,
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

fn try_create_collection_1155(
    deps: DepsMut,
    _env: Env,
    name: String,
    symbol: String,
    has_admin: bool,
    admin: Option<Addr>,
    curators: Vec<Addr>,
    initial_tokens: Vec<CurateTokenId>,
    entropy: String,
    label: String,
    source_nft_contract_address: String,
    source_chain: String,
    destination_user_address: Addr,
    token_id: String,
    token_amount: u128,
    royalty: u16,
    royalty_receiver: Addr,
    metadata: String,
    transaction_hash:String
) -> Result<Response, ContractError> {
    // let owner = admin.clone().unwrap().into_string();
    // let owner_addr = deps.api.addr_validate(&owner)?;

    // let factory = ContractInfo {
    //     code_hash: env.contract.code_hash,
    //     address: env.contract.address,
    // };

    let initmsg = Collection1155InstantiateMsg {
        has_admin,
        admin,
        curators,
        initial_tokens,
        entropy,
        label,
        source_nft_contract_address,
        source_chain,
        destination_user_address,
        token_id,
        token_amount,
        royalty,
        royalty_receiver,
        metadata,
        transaction_hash
    };

    let offspring_code = SNIP1155_CODE.load(deps.storage)?;
    let init_submsg = SubMsg::reply_always(
        initmsg.to_cosmos_msg(
            None,
            name + &symbol,
            offspring_code.code_id,
            offspring_code.code_hash,
            None,
        )?,
        SNIP1155_INSTANTIATE_REPLY_ID,
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
        SNIP721_INSTANTIATE_REPLY_ID => handle_instantiate_reply_721(msg),
        SNIP1155_INSTANTIATE_REPLY_ID => handle_instantiate_reply_1155(msg),
        id => Err(ContractError::UnexpectedReplyId { id }),
    }
}

fn handle_instantiate_reply_721(msg: Reply) -> Result<Response, ContractError> {
    // The parsing process below can be handled easier if one imports cw-plus
    // See: https://github.com/CosmWasm/cw-plus/blob/main/packages/utils/src/parse_reply.rs
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyCollectionInfo = from_binary(&bin)?;
                register_collection_721_impl(reply_info)
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
                let reply_info: ReplyCollectionInfo = from_binary(&bin)?;
                register_collection_1155_impl(reply_info)
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
fn register_collection_721_impl(
    reply_info: ReplyCollectionInfo,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_attribute("collection_address_721", &reply_info.address)
        .set_data(to_binary(&reply_info)?))
}

fn register_collection_1155_impl(
    reply_info: ReplyCollectionInfo,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_attribute("collection_address_1155", &reply_info.address)
        .set_data(to_binary(&reply_info)?))
}
