use std::collections::BTreeMap;

use collection_deployer::bridge_msg::ReplyCollectionDeployerInfo;
use common::CodeInfo;
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Api, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, StdResult, Storage, SubMsg, SubMsgResult, Uint128, Uint256, WasmMsg
};
use schemars::JsonSchema;
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::{Keymap, WithoutIter};
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};
use snip1155::reply::ReplyCollectionInfo as ReplyCollection1155Info;
use snip1155::state::metadata::Metadata as Snip1155Meta;
use snip1155::state::state_structs::{
    CurateTokenId, LbPair, TknConfig, TokenAmount, TokenIdBalance, TokenInfoMsg
};
use snip721::reply::ReplyCollectionInfo as ReplyCollection721Info;
use snip721::royalties::{Royalty, RoyaltyInfo};
use snip721::token::Metadata as Snip721Meta;
use storage_deployer::bridge_msg::ReplyStorageDeployerInfo;
use storage_deployer::structs::{ReplyStorage721Info,ReplyStorage1155Info};

use crate::error::ContractError;
use crate::events::{
    AddNewValidatorEventInfo, Claimed1155EventInfo, Claimed721EventInfo, LockedEventInfo,
    RewardValidatorEventInfo, UnLock1155EventInfo, UnLock721EventInfo,
};
use crate::msg::{BlacklistValidatorMsg, BridgeExecuteMsg, BridgeQueryAnswer, BridgeQueryMsg};
use crate::state::{
    config, config_read, BLACKLISTED_VALIDATORS, CODEHASHES, COLLECTION_DEPLOYER_1155_REPLY_ID,
    COLLECTION_DEPLOYER_721_REPLY_ID, COLLECTION_DEPLOYER_REPLY_ID, COLLETION_DEPLOYER_CODE,
    DUPLICATE_STORAGE_1155, DUPLICATE_STORAGE_721, DUPLICATE_TO_ORIGINAL_STORAGE,
    NFT_COLLECTION_OWNER, ORIGINAL_STORAGE_1155, ORIGINAL_STORAGE_721,
    ORIGINAL_TO_DUPLICATE_STORAGE, STORAGE_DEPLOYER_1155_REPLY_ID, STORAGE_DEPLOYER_721_REPLY_ID,
    STORAGE_DEPLOYER_CODE, STORAGE_DEPLOYER_REPLY_ID, UNIQUE_IDENTIFIER_STORAGE,
    VALIDATORS_STORAGE,
};
use crate::structs::{
    AddValidatorMsg, BridgeInstantiateMsg, ClaimData, ClaimMsg, ClaimValidatorRewardsMsg,
    DuplicateToOriginalContractInfo, Lock1155Msg, Lock721Msg, OriginalToDuplicateContractInfo,
    SignerAndSignature, State, Validator, VerifyMsg,
};
use sha2::{Digest, Sha256};
use snip1155::msg::{Snip1155ExecuteMsg,Snip1155QueryMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
struct Balance {
    amount: Uint128,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: BridgeInstantiateMsg,
) -> StdResult<Response> {
    let mut validators_count = 0;
    for val in msg.validators {
        validators_count += 1;
        VALIDATORS_STORAGE.insert(
            deps.storage,
            &val.0,
            &Validator {
                address: val.1,
                added: true,
                pending_reward: 0,
            },
        )?;
    }

    let state = State {
        collection_deployer: _env.contract.address.clone(),
        storage_deployer: _env.contract.address.clone(),
        validators_count,
        self_chain: msg.chain_type,
        type_erc_721: "singular".to_owned(),
        type_erc_1155: "multiple".to_owned(),
    };

    config(deps.storage).save(&state)?;

    STORAGE_DEPLOYER_CODE.save(deps.storage, &msg.storage_deployer_code_info)?;
    COLLETION_DEPLOYER_CODE.save(deps.storage, &msg.collection_deployer_code_info)?;

    let init_storage_deployer_msg = storage_deployer::msg::StorageDeployerInstantiateMsg {
        storage721_code_info: msg.storage721_code_info,
        storage1155_code_info: msg.storage1155_code_info,
    };

    let init_storage_deployer_sub_msg = SubMsg::reply_always(
        init_storage_deployer_msg.to_cosmos_msg(
            None,
            msg.storage_label,
            msg.storage_deployer_code_info.code_id,
            msg.storage_deployer_code_info.code_hash,
            None,
        )?,
        STORAGE_DEPLOYER_REPLY_ID,
    );

    let init_collection_deployer_msg = collection_deployer::msg::CollectionDeployerInstantiateMsg {
        collection721_code_info: msg.collection721_code_info,
        collection1155_code_info: msg.collection1155_code_info,
    };

    let init_collection_deployer_submsg = SubMsg::reply_always(
        init_collection_deployer_msg.to_cosmos_msg(
            None,
            msg.collection_label,
            msg.collection_deployer_code_info.code_id,
            msg.collection_deployer_code_info.code_hash,
            None,
        )?,
        COLLECTION_DEPLOYER_REPLY_ID,
    );
    Ok(Response::new()
        .add_submessages(vec![
            init_storage_deployer_sub_msg,
            init_collection_deployer_submsg,
        ])
        .add_attribute("bridge_contract_address", _env.contract.address))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BridgeExecuteMsg,
) -> StdResult<Response> {
    match msg {
        BridgeExecuteMsg::AddValidator { data } => add_validator(deps, data),
        BridgeExecuteMsg::ClaimValidatorRewards { data } => claim_validator_rewards(deps, data),
        BridgeExecuteMsg::BlacklistValidator { data } => blacklist_validator(deps, data),
        BridgeExecuteMsg::Lock721 { data } => lock721(deps, env, data),
        BridgeExecuteMsg::Lock1155 { data } => lock1155(deps, env, info, data),
        BridgeExecuteMsg::Claim721 { data } => claim721(deps, env, info, data),
        BridgeExecuteMsg::Claim1155 { data } => claim1155(deps, env, info, data),
        BridgeExecuteMsg::VerifySig { data } => verify_sig(deps, data),
    }
}

fn verify_sig(deps: DepsMut, msg: VerifyMsg) -> StdResult<Response> {
    let serialized = serde_json::to_vec(&msg.claim_data_as_binary)
        .expect("Failed to convert claim data to binary");
    let mut hasher = Sha256::new();
    hasher.update(serialized);

    let serialized = msg.claim_data_as_binary.concat_all_fields();
    let mut hasher = Sha256::new();

    hasher.update(serialized);

    let serialized = msg.msg_as_bindary;
    let mut hasher = Sha256::new();

    hasher.update(&*serialized);

    let res = deps
        .api
        .secp256k1_verify(&msg.claim_data, &msg.signature, &msg.user)?;
    if !res {
        Err(StdError::generic_err(
            "Signature verification unsuccessful".to_string(),
        ))
    } else {
        Ok(Response::default())
    }
}

fn matches_current_chain(storage: &dyn Storage, destination_chain: String) -> StdResult<Response> {
    if destination_chain != config_read(storage).load()?.self_chain {
        return Err(StdError::generic_err("Invalid destination chain!"));
    } else {
        Ok(Response::default())
    }
}

fn has_correct_fee(fee: u128, info: MessageInfo) -> StdResult<Response> {
    let msg_value = info.funds[0].amount;

    if msg_value >= Uint128::from(fee) {
        Ok(Response::default())
    } else {
        return Err(StdError::generic_err("data.fee LESS THAN sent amount!"));
    }
}

fn add_validator(deps: DepsMut, add_validator_msg: AddValidatorMsg) -> StdResult<Response> {
    if add_validator_msg.signatures.is_empty() {
        return Err(StdError::generic_err("Must have signatures!"));
    }
    if BLACKLISTED_VALIDATORS.contains(deps.storage, &add_validator_msg.validator.0.clone()) {
        return Err(StdError::generic_err("Validator is blacklisted"));
    }
    let state = config_read(deps.storage).load()?;
    if VALIDATORS_STORAGE.contains(deps.storage, &add_validator_msg.validator.0) {
        return Err(StdError::generic_err("Validator already added"));
    }
    let required = required_threshold(state.validators_count as u128) as u128;

    let serialized = add_validator_msg.validator.0.to_string();
    let mut hasher = Sha256::new();
    let mut hash = [0u8; 32];
    hasher.update(serialized);
    hash = hasher.finalize().into();

    let percentage = validate_signature(
        deps.api,
        hash,
        add_validator_msg.signatures,
        required,
    )?;
    if percentage.len() < required as usize {
        return Err(StdError::generic_err("Threshold not reached!"));
    }

    Ok(add_validator_to_state(
        deps.storage,
        &add_validator_msg.validator,
    )?)
}

fn blacklist_validator(deps: DepsMut, blacklist_msg: BlacklistValidatorMsg) -> StdResult<Response> {
    if blacklist_msg.signatures.len() <= 0 {
        return Err(StdError::generic_err("Must have signatures!"));
    }
    if !VALIDATORS_STORAGE.contains(deps.storage, &blacklist_msg.validator.0) {
        return Err(StdError::generic_err("Validator is not added"));
    }
    let state = config_read(deps.storage).load()?;
    let req = required_threshold(state.validators_count as u128) as u128;
    let percentage = validate_signature(
        deps.api,
        blacklist_msg.validator.0 .0.clone().try_into().unwrap(),
        blacklist_msg.signatures,
        req,
    )?;

    if percentage.len() < req as usize {
        return Err(StdError::generic_err("Threshold not reached!"));
    }
    VALIDATORS_STORAGE.remove(deps.storage, &blacklist_msg.validator.0)?;

    config(deps.storage).update(|mut state| {
        state.validators_count -= 1;
        Result::<State, StdError>::Ok(state) // Return the modified state
    })?;
    BLACKLISTED_VALIDATORS.insert(deps.storage, &blacklist_msg.validator.0, &true)?;
    Ok(Response::new())
}

fn verify_signature(
    api: &dyn Api,
    signature: &[u8],
    signer_address: &[u8],
    hash: &[u8; 32],
) -> StdResult<bool> {
    let verify = api.secp256k1_verify(hash, &signature, &signer_address);
    match verify {
        Ok(v) => {
            Ok(v)
        },
        Err(_) => {
            Ok(false)
        },
    }
}

fn required_threshold(validators_count: u128) -> i128 {
    ((validators_count * 2) / 3) as i128 + 1
}

fn add_validator_to_state(
    storage: &mut dyn Storage,
    validator: &(Binary, Addr),
) -> StdResult<Response> {
    VALIDATORS_STORAGE.insert(
        storage,
        &validator.0,
        &Validator {
            address: validator.1.clone(),
            added: true,
            pending_reward: 0,
        },
    )?;
    config(storage).update(|mut state| -> Result<_, StdError> {
        state.validators_count += 1;
        Ok(state) // Return the modified state
    })?;

    let log: Vec<Attribute> = vec![AddNewValidatorEventInfo::new(validator.1.clone()).try_into()?];

    Ok(Response::new().add_attributes(log)) // Indicate successful completion of the function
}

fn claim_validator_rewards(deps: DepsMut, data: ClaimValidatorRewardsMsg) -> StdResult<Response> {
    if !VALIDATORS_STORAGE
        .get(deps.storage, &data.validator)
        .is_some()
    {
        return Err(StdError::generic_err("Validator does not exist!"));
    }

    let rewards_option = VALIDATORS_STORAGE.get(deps.storage, &data.validator.clone());
    let res;
    match rewards_option {
        Some(mut v) => {
            let coins_to_send: Vec<Coin> = vec![Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(v.pending_reward),
            }];

            let message = CosmosMsg::<Empty>::Bank(BankMsg::Send {
                // replace with recipient of your choice
                to_address: v.address.clone().into_string(),
                amount: coins_to_send,
            });

            v.pending_reward = 0;

            let _ = VALIDATORS_STORAGE.insert(deps.storage, &data.validator, &v);

            let log: Vec<Attribute> = vec![RewardValidatorEventInfo::new(v.address).try_into()?];

            res = Response::new().add_message(message).add_attributes(log);
        }
        None => todo!(),
    }

    Ok(res)
}

fn check_storage_721(
    deps: DepsMut,
    self_chain: String,
    storage_mapping_721: &Keymap<'static, (String, String), (Addr, String), Bincode2, WithoutIter>,
    source_nft_contract_address: Addr,
    token_id: String,
    collection_code_info: CodeInfo,
    owner: Addr,
    is_original: bool,
    time: u64
) -> StdResult<Response> {
    let storage_address_option = storage_mapping_721.get(
        deps.storage,
        &(
            source_nft_contract_address.clone().into_string(),
            self_chain.clone(),
        ),
    );


    let _ = CODEHASHES.insert(
        deps.storage,
        &source_nft_contract_address.clone(),
        &collection_code_info.code_hash.clone(),
    );

    match storage_address_option {
        Some(v) => transfer_to_storage_721(
            deps.storage,
            v.0,
            source_nft_contract_address.clone(),
            token_id,
            collection_code_info.code_hash,
        ),
        None => {
            let create_storage_msg =
                storage_deployer::msg::StorageDeployerExecuteMsg::CreateStorage721 {
                    label: source_nft_contract_address.clone().into_string() + &time.to_string(),
                    collection_address: source_nft_contract_address.clone(),
                    collection_code_info,
                    owner: owner.into_string(),
                    is_original,
                    token_id,
                    source_nft_contract_address
                };

            let code_info = STORAGE_DEPLOYER_CODE.load(deps.storage)?;

            let init_submsg = SubMsg::reply_always(
                create_storage_msg.to_cosmos_msg(
                    code_info.code_hash,
                    config(deps.storage).load()?.storage_deployer.to_string(),
                    None,
                )?,
                STORAGE_DEPLOYER_721_REPLY_ID,
            );
            Ok(Response::new().add_submessage(init_submsg))
        }
    }
}

fn transfer_to_storage_721(
    storage: &mut dyn Storage,
    storage_address: Addr,
    source_nft_contract_address: Addr,
    token_id: String,
    code_hash: String,
) -> StdResult<Response> {
    let transfer_msg = snip721::msg::Snip721ExecuteMsg::TransferNft {
        recipient: storage_address.clone().into_string(),
        token_id: token_id.to_string(),
        memo: Option::None,
        padding: Option::None,
    }
    .to_cosmos_msg(
        code_hash,
        source_nft_contract_address.clone().into_string(),
        None,
    )?;

    let _ = NFT_COLLECTION_OWNER.insert(
        storage,
        &(
            source_nft_contract_address.clone().into_string(),
            token_id.to_string(),
        ),
        &(storage_address, 1u128),
    );

    Ok(Response::new().add_message(transfer_msg))
}

fn lock721(deps: DepsMut, env: Env, msg: Lock721Msg) -> StdResult<Response> {
    let addr_result = deps
        .api
        .addr_validate(&msg.source_nft_contract_address.clone().into_string());
    let self_chain = config(deps.storage).load()?.self_chain.clone();
    match addr_result {
        Ok(_v) => {}
        Err(_) => {
            return Err(StdError::generic_err(
                "sourceNftContractAddress cannot be zero address",
            ));
        }
    }

    let original_collection_address_option = DUPLICATE_TO_ORIGINAL_STORAGE.get(
        deps.storage,
        &(msg.source_nft_contract_address.clone(), self_chain.clone()),
    );

    match original_collection_address_option {
        Some(_v) => {
            // notOriginal
            let log: Vec<Attribute> = vec![LockedEventInfo::new(
                msg.token_id.clone(),
                msg.destination_chain,
                msg.destination_user_address,
                _v.contract_address,
                1,
                config_read(deps.storage).load()?.type_erc_721,
                _v.chain,
                msg.metadata_uri,
            )
            .try_into()?];

            let res = check_storage_721(
                deps,
                self_chain.clone(),
                &DUPLICATE_STORAGE_721,
                msg.source_nft_contract_address.clone(),
                msg.token_id,
                msg.collection_code_info,
                env.contract.address,
                false,
                env.block.time.seconds()
            )?;

            Ok(res.add_attributes(log))
        }
        None => {
            // isOriginal

            let log: Vec<Attribute> = vec![LockedEventInfo::new(
                msg.token_id.clone(),
                msg.destination_chain.clone(),
                msg.destination_user_address,
                msg.source_nft_contract_address.to_string(),
                1,
                config_read(deps.storage).load()?.type_erc_721,
                config(deps.storage).load()?.self_chain,
                msg.metadata_uri,
            )
            .try_into()?];

            let res = check_storage_721(
                deps,
                self_chain,
                &ORIGINAL_STORAGE_721,
                msg.source_nft_contract_address.clone(),
                msg.token_id,
                msg.collection_code_info,
                env.contract.address,
                true,
                env.block.time.seconds()
            )?;

            Ok(res.add_attributes(log))
        }
    }
}

fn check_storage_1155(
    deps: DepsMut,
    self_chain: String,
    storage_mapping_1155: &Keymap<'static, (String, String), (Addr, String), Bincode2, WithoutIter>,
    source_nft_contract_address: Addr,
    token_id: String,
    token_amount: u128,
    collection_code_info: CodeInfo,
    owner: Addr,
    is_original: bool,
    from: Addr,
    time: u64,
) -> StdResult<Response> {
    let storage_address_option = storage_mapping_1155.get(
        deps.storage,
        &(
            source_nft_contract_address.clone().into_string(),
            self_chain,
        ),
    );


    let _ = CODEHASHES.insert(
        deps.storage,
        &source_nft_contract_address.clone(),
        &collection_code_info.code_hash.clone(),
    );

    match storage_address_option {
        Some(v) => transfer_to_storage_1155(
            deps.storage,
            deps.api,
            from,
            v.0,
            source_nft_contract_address.clone(),
            token_id,
            token_amount,
            collection_code_info.code_hash,
        ),
        None => {
            let create_storage_msg =
                storage_deployer::msg::StorageDeployerExecuteMsg::CreateStorage1155 {
                    label: source_nft_contract_address.clone().into_string() + &time.to_string(),
                    collection_address: source_nft_contract_address.clone(),
                    collection_code_info,
                    owner: owner.into_string(),
                    is_original,
                    token_id,
                    token_amount,
                    from,
                    source_nft_contract_address
                };

            let code_info = STORAGE_DEPLOYER_CODE.load(deps.storage)?;

            let init_submsg = SubMsg::reply_always(
                create_storage_msg.to_cosmos_msg(
                    code_info.code_hash,
                    config(deps.storage).load()?.storage_deployer.to_string(),
                    None,
                )?,
                STORAGE_DEPLOYER_1155_REPLY_ID,
            );
            Ok(Response::new().add_submessage(init_submsg))
        }
    }
}

fn transfer_to_storage_1155(
    storage: &mut dyn Storage,
    api: &dyn Api,
    from: Addr,
    storage_address: Addr,
    source_nft_contract_address: Addr,
    token_id: String,
    token_amount: u128,
    code_hash: String,
) -> StdResult<Response> {

    api
        .debug(format!("transfer_to_storage_1155 start").as_str());

    let transfer_msg = snip1155::msg::Snip1155ExecuteMsg::Transfer {
        token_id: token_id.to_string(),
        from,
        recipient: storage_address.clone(),
        amount: Uint256::from(token_amount),
        memo: Option::None,
        padding: Option::None,
    }
    .to_cosmos_msg(
        code_hash,
        source_nft_contract_address.clone().into_string(),
        None,
    )?;

    api
        .debug(format!("transfer_to_storage_1155 mid").as_str());

    let _ = NFT_COLLECTION_OWNER.insert(
        storage,
        &(
            source_nft_contract_address.clone().into_string(),
            token_id.to_string(),
        ),
        &(storage_address, token_amount),
    );

    api
        .debug(format!("transfer_to_storage_1155 end").as_str());

    Ok(Response::new().add_message(transfer_msg))
}

fn lock1155(deps: DepsMut, env: Env, info: MessageInfo, msg: Lock1155Msg) -> StdResult<Response> {
    let addr_result = deps
        .api
        .addr_validate(&msg.source_nft_contract_address.clone().into_string());
    let self_chain = config(deps.storage).load()?.self_chain.clone();
    match addr_result {
        Ok(_v) => {}
        Err(_) => {
            return Err(StdError::generic_err(
                "sourceNftContractAddress cannot be zero address",
            ));
        }
    }

    if msg.token_amount <= 0 {
        return Err(StdError::generic_err("token amount must be > than zero"));
    }

    let original_collection_address_option = DUPLICATE_TO_ORIGINAL_STORAGE.get(
        deps.storage,
        &(msg.source_nft_contract_address.clone(), self_chain.clone()),
    );

    match original_collection_address_option {
        Some(_v) => {
            // notOriginal

            let log: Vec<Attribute> = vec![LockedEventInfo::new(
                msg.token_id.clone(),
                msg.destination_chain,
                msg.destination_user_address,
                _v.contract_address,
                msg.token_amount,
                config_read(deps.storage).load()?.type_erc_1155,
                _v.chain,
                msg.metadata_uri,
            )
            .try_into()?];

            let res = check_storage_1155(
                deps,
                self_chain.clone(),
                &DUPLICATE_STORAGE_1155,
                msg.source_nft_contract_address.clone(),
                msg.token_id,
                msg.token_amount,
                msg.collection_code_info,
                env.contract.address,
                false,
                info.sender.clone(),
                env.block.time.seconds(),
            )?;

            Ok(res.add_attributes(log))
        }
        None => {
            // isOriginal

            let log: Vec<Attribute> = vec![LockedEventInfo::new(
                msg.token_id.clone(),
                msg.destination_chain.clone(),
                msg.destination_user_address,
                msg.source_nft_contract_address.to_string(),
                msg.token_amount,
                config_read(deps.storage).load()?.type_erc_1155,
                config(deps.storage).load()?.self_chain,
                msg.metadata_uri,
            )
            .try_into()?];

            let res = check_storage_1155(
                deps,
                self_chain,
                &ORIGINAL_STORAGE_1155,
                msg.source_nft_contract_address.clone(),
                msg.token_id,
                msg.token_amount,
                msg.collection_code_info,
                env.contract.address,
                true,
                info.sender.clone(),
                env.block.time.seconds(),
            )?;
            Ok(res.add_attributes(log))
        }
    }
}

fn create_claim_data_hash(data: ClaimData) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data.concat_all_fields());
    hasher.finalize().into()
}
fn validate_signature(
    api: &dyn Api,
    hash: [u8; 32],
    signatures: Vec<SignerAndSignature>,
    validators_count: u128,
) -> StdResult<Vec<Binary>> {
    let mut percentage = 0;
    let mut uv: BTreeMap<Binary, bool> = BTreeMap::new();
    for ele in signatures {
        if uv.contains_key(&ele.signer_address) {
            continue;
        }
        if verify_signature(api, &ele.signature, &ele.signer_address, &hash)? {
            percentage += 1;
            uv.insert(ele.signer_address, true);
        }
    }
    if percentage < required_threshold(validators_count) {
        return Err(StdError::generic_err("Threshold not reached!"));
    }
    Ok(uv.keys().cloned().collect())
}

fn reward_validators(
    storage: &mut dyn Storage,
    fee: u128,
    validators_to_reward: Vec<Binary>,
    balance: u128,
) -> StdResult<()> {
    if fee <= 0 {
        return Err(StdError::generic_err("Invalid fees"));
    }
    if balance >= fee{
    }
    else{
        return Err(StdError::generic_err("No rewards available"));
    }
    let fee_per_validator = fee / validators_to_reward.len() as u128;

    for val in validators_to_reward {
        let mut validator_option = VALIDATORS_STORAGE
            .get(storage, &val)
            .expect("Unreachable: Validator not found found");

        validator_option.pending_reward = validator_option.pending_reward + fee_per_validator;

        VALIDATORS_STORAGE.insert(storage, &val, &validator_option)?;
    }
    Ok(())
}

fn deploy_collection_721(
    deps: DepsMut,
    name: String,
    symbol: String,
    owner: String,
    source_nft_contract_address: String,
    source_chain: String,
    destination_user_address: Addr,
    token_id: String,
    token_amount: u128,
    royalty: u16,
    royalty_receiver: Addr,
    metadata: String,
    transaction_hash: String,
    lock_tx_chain: String,
    time: u64
) -> StdResult<Response> {
    let create_collection_msg =
        collection_deployer::msg::CollectionDeployerExecuteMsg::CreateCollection721 {
            label: name.clone() + &symbol + &source_nft_contract_address + &time.to_string(),
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
            lock_tx_chain,
        };

    let code_info = COLLETION_DEPLOYER_CODE.load(deps.storage)?;

    let init_submsg = SubMsg::reply_always(
        create_collection_msg.to_cosmos_msg(
            code_info.code_hash,
            config(deps.storage).load()?.collection_deployer.to_string(),
            None,
        )?,
        COLLECTION_DEPLOYER_721_REPLY_ID,
    );
    Ok(Response::new().add_submessage(init_submsg))
}

fn deploy_collection_1155(
    deps: DepsMut,
    name: String,
    symbol: String,
    owner: Addr,
    source_nft_contract_address: String,
    source_chain: String,
    destination_user_address: Addr,
    token_id: String,
    token_amount: u128,
    royalty: u16,
    royalty_receiver: Addr,
    metadata: String,
    transaction_hash: String,
    lock_tx_chain: String,
    time: u64
) -> StdResult<Response> {
    let create_collection_msg =
        collection_deployer::msg::CollectionDeployerExecuteMsg::CreateCollection1155 {
            has_admin: true,
            admin: Some(owner.clone()),
            curators: vec![owner.clone()],
            initial_tokens: vec![CurateTokenId {
                token_info: TokenInfoMsg {
                    token_id: token_id.to_string(),
                    name: name.clone(),
                    symbol: symbol.clone(),
                    token_config: TknConfig::Fungible {
                        minters: vec![owner.clone()],
                        decimals: 6,
                        public_total_supply: true,
                        enable_mint: true,
                        enable_burn: true,
                        minter_may_update_metadata: true,
                    },
                    public_metadata: Some(Snip1155Meta {
                        token_uri: Some(metadata.clone()),
                        extension: Option::None,
                    }),
                    private_metadata: Option::None,
                },
                balances: vec![TokenIdBalance {
                    address: destination_user_address.clone(),
                    amount: Uint256::from(token_amount),
                }],
            }],
            entropy: name.clone() + &symbol + &source_nft_contract_address,
            lb_pair_info: LbPair{
                name: name.clone(),
                symbol: symbol.clone(),
                lb_pair_address: Addr::unchecked(&source_nft_contract_address),
                decimals: 18,
            },
            label: name.clone() + &symbol + &source_nft_contract_address + &time.to_string(),
            source_nft_contract_address,
            source_chain,
            destination_user_address,
            token_id,
            token_amount,
            royalty,
            royalty_receiver,
            metadata,
            name,
            symbol,
            transaction_hash,
            lock_tx_chain,
        };

    let code_info = COLLETION_DEPLOYER_CODE.load(deps.storage)?;
    let init_submsg = SubMsg::reply_always(
        create_collection_msg.to_cosmos_msg(
            code_info.code_hash,
            config(deps.storage).load()?.collection_deployer.to_string(),
            None,
        )?,
        COLLECTION_DEPLOYER_1155_REPLY_ID,
    );
    Ok(Response::new().add_submessage(init_submsg))
}

fn claim721(deps: DepsMut, env: Env, info: MessageInfo, msg: ClaimMsg) -> StdResult<Response> {
    let balance = deps
        .querier
        .query_balance(env.contract.address.clone(), "uscrt".to_string())?
        .amount;
    let self_chain = config_read(deps.storage).load()?.self_chain;

    let _ = has_correct_fee(msg.data.fee.clone(), info)?;

    let type_erc_721 = config_read(deps.storage).load()?.type_erc_721;

    let validators_count = config_read(deps.storage).load()?.validators_count;

    let _ = matches_current_chain(deps.storage, msg.data.destination_chain.clone())?;

    if msg.data.nft_type != type_erc_721 {
        return Err(StdError::generic_err("Invalid NFT type!"));
    }

    let hash = create_claim_data_hash(msg.data.clone());

    let exists = UNIQUE_IDENTIFIER_STORAGE.contains(deps.storage, &hash);
    if exists {
        return Err(StdError::generic_err("Data already processed!"));
    }
    UNIQUE_IDENTIFIER_STORAGE.insert(deps.storage, &hash, &true)?;

    let validators_to_reward = validate_signature(
        deps.api,
        hash,
        msg.signatures,
        validators_count.try_into().unwrap(),
    )?;

    reward_validators(
        deps.storage,
        msg.data.fee.clone(),
        validators_to_reward,
        balance.into(),
    )?;

    let duplicate_collection_address_option = ORIGINAL_TO_DUPLICATE_STORAGE.get(
        deps.storage,
        &(
            msg.data.source_nft_contract_address.clone(),
            msg.data.source_chain.clone(),
        ),
    );
    let mut duplicate_collection_address = OriginalToDuplicateContractInfo {
        chain: "".to_string(),
        contract_address: env.contract.address.clone(),
        code_hash: "".to_string(),
    };
    let mut has_duplicate: bool = false;
    let mut has_storage: bool = false;
    let storage_contract_option: Option<(Addr, String)>;
    let storage_contract: (Addr, String);

    match duplicate_collection_address_option {
        Some(v) => {
            duplicate_collection_address = v;
            storage_contract_option = DUPLICATE_STORAGE_721.get(
                deps.storage,
                &(
                    duplicate_collection_address.contract_address.to_string(),
                    self_chain,
                ),
            );
            has_duplicate = true
        }
        None => {
            storage_contract_option = ORIGINAL_STORAGE_721.get(
                deps.storage,
                &(msg.data.source_nft_contract_address.clone(), self_chain),
            );
        }
    }
    match storage_contract_option {
        Some(v) => {
            storage_contract = v;
            has_storage = true;
        }
        None => {
            storage_contract = (Addr::unchecked("none"), "none".to_string());
            has_storage = false;
        }
    }
    let res = if has_duplicate && has_storage {
        let is_storage_is_nft_owner_option = NFT_COLLECTION_OWNER.get(
            deps.storage,
            &(
                duplicate_collection_address
                    .contract_address
                    .clone()
                    .into_string(),
                msg.data.token_id.to_string(),
            ),
        );

        match is_storage_is_nft_owner_option {
            Some(_v) => {
                let create_unlock_msg = storage721::msg::Storage721ExecuteMsg::UnLockToken {
                    token_id: msg.data.token_id.clone(),
                    to: msg.data.destination_user_address.clone(),
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: storage_contract.0.clone().into_string(),
                    code_hash: storage_contract.1,
                    msg: to_binary(&create_unlock_msg)?,
                    funds: vec![],
                });

                let log: Vec<Attribute> = vec![
                    UnLock721EventInfo::new(
                        msg.data.destination_user_address,
                        msg.data.token_id.clone(),
                        storage_contract.0.to_string(),
                    )
                    .try_into()?,
                    Claimed721EventInfo::new(
                        msg.data.lock_tx_chain,
                        msg.data.source_chain,
                        msg.data.transaction_hash,
                        duplicate_collection_address
                            .contract_address
                            .clone()
                            .to_string(),
                        msg.data.token_id.clone(),
                    )
                    .try_into()?,
                ];

                let _ = NFT_COLLECTION_OWNER.remove(
                    deps.storage,
                    &(
                        duplicate_collection_address.contract_address.into_string(),
                        msg.data.token_id,
                    ),
                );

                Ok(Response::new().add_message(message).add_attributes(log))
            }
            None => {
                let create_collection_msg = snip721::msg::Snip721ExecuteMsg::MintNft {
                    token_id: Some(msg.data.token_id.to_string()),
                    owner: Some(msg.data.destination_user_address.into_string()),
                    public_metadata: Some(Snip721Meta {
                        token_uri: Some(msg.data.metadata),
                        extension: Option::None,
                    }),
                    private_metadata: Option::None,
                    serial_number: Option::None,
                    royalty_info: Some(RoyaltyInfo {
                        decimal_places_in_rates: 4,
                        royalties: vec![Royalty {
                            rate: msg.data.royalty,
                            recipient: msg.data.royalty_receiver.into_string(),
                        }],
                    }),
                    transferable: Some(true),
                    memo: Option::None,
                    padding: Option::None,
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: duplicate_collection_address
                        .contract_address
                        .clone()
                        .into_string(),
                    code_hash: duplicate_collection_address.code_hash,
                    msg: to_binary(&create_collection_msg)?,
                    funds: vec![],
                });
                let log: Vec<Attribute> = vec![Claimed721EventInfo::new(
                    msg.data.lock_tx_chain,
                    msg.data.source_chain,
                    msg.data.transaction_hash,
                    duplicate_collection_address.contract_address.to_string(),
                    msg.data.token_id,
                )
                .try_into()?];

                Ok(Response::new().add_message(message).add_attributes(log))
            }
        }
    }
    // ===============================/ hasDuplicate && NOT hasStorage /=======================
    else if has_duplicate && !has_storage {
        let create_collection_msg = snip721::msg::Snip721ExecuteMsg::MintNft {
            token_id: Some(msg.data.token_id.to_string()),
            owner: Some(msg.data.destination_user_address.into_string()),
            public_metadata: Some(Snip721Meta {
                token_uri: Some(msg.data.metadata),
                extension: Option::None,
            }),
            private_metadata: Option::None,
            serial_number: Option::None,
            royalty_info: Some(RoyaltyInfo {
                decimal_places_in_rates: 4,
                royalties: vec![Royalty {
                    rate: msg.data.royalty,
                    recipient: msg.data.royalty_receiver.into_string(),
                }],
            }),
            transferable: Some(true),
            memo: Option::None,
            padding: Option::None,
        };

        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: duplicate_collection_address
                .contract_address
                .clone()
                .into_string(),
            code_hash: duplicate_collection_address.code_hash,
            msg: to_binary(&create_collection_msg)?,
            funds: vec![],
        });
        let log: Vec<Attribute> = vec![Claimed721EventInfo::new(
            msg.data.lock_tx_chain,
            msg.data.source_chain,
            msg.data.transaction_hash,
            duplicate_collection_address.contract_address.to_string(),
            msg.data.token_id,
        )
        .try_into()?];

        Ok(Response::new().add_message(message).add_attributes(log))
    }
    // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
    else if !has_duplicate && !has_storage {
        // new collection
        deploy_collection_721(
            deps,
            msg.data.name,
            msg.data.symbol,
            env.contract.address.into_string(),
            msg.data.source_nft_contract_address,
            msg.data.source_chain.clone(),
            msg.data.destination_user_address,
            msg.data.token_id,
            1,
            msg.data.royalty,
            msg.data.royalty_receiver,
            msg.data.metadata,
            msg.data.transaction_hash,
            msg.data.lock_tx_chain,
            env.block.time.seconds(),
        )
    }
    // ===============================/ NOT hasDuplicate && hasStorage /=======================
    else if !has_duplicate && has_storage {
        let code_hash = CODEHASHES
            .get(
                deps.storage,
                &deps
                    .api
                    .addr_validate(&msg.data.source_nft_contract_address.clone())?,
            )
            .expect("Code hash not found");

        let is_storage_is_nft_owner_option = NFT_COLLECTION_OWNER.get(
            deps.storage,
            &(
                msg.data.source_nft_contract_address.clone(),
                msg.data.token_id.to_string(),
            ),
        );

        match is_storage_is_nft_owner_option {
            Some(_v) => {
                let create_unlock_msg = storage721::msg::Storage721ExecuteMsg::UnLockToken {
                    token_id: msg.data.token_id.clone(),
                    to: msg.data.destination_user_address.clone(),
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: storage_contract.0.clone().into_string(),
                    code_hash: storage_contract.1,
                    msg: to_binary(&create_unlock_msg)?,
                    funds: vec![],
                });

                let log: Vec<Attribute> = vec![
                    UnLock721EventInfo::new(
                        msg.data.destination_user_address,
                        msg.data.token_id.clone(),
                        storage_contract.0.to_string(),
                    )
                    .try_into()?,
                    Claimed721EventInfo::new(
                        msg.data.lock_tx_chain,
                        msg.data.source_chain,
                        msg.data.transaction_hash,
                        msg.data.source_nft_contract_address.clone(),
                        msg.data.token_id.clone(),
                    )
                    .try_into()?,
                ];

                let _ = NFT_COLLECTION_OWNER.remove(
                    deps.storage,
                    &(
                        msg.data.source_nft_contract_address.clone(),
                        msg.data.token_id,
                    ),
                );

                Ok(Response::new().add_message(message).add_attributes(log))
            }
            None => {
                // CANT BE THERE
                let create_collection_msg = snip721::msg::Snip721ExecuteMsg::MintNft {
                    token_id: Some(msg.data.token_id.to_string()),
                    owner: Some(msg.data.destination_user_address.into_string()),
                    public_metadata: Some(Snip721Meta {
                        token_uri: Some(msg.data.metadata),
                        extension: Option::None,
                    }),
                    private_metadata: Option::None,
                    serial_number: Option::None,
                    royalty_info: Some(RoyaltyInfo {
                        decimal_places_in_rates: 4,
                        royalties: vec![Royalty {
                            rate: msg.data.royalty,
                            recipient: msg.data.royalty_receiver.into_string(),
                        }],
                    }),
                    transferable: Some(true),
                    memo: Option::None,
                    padding: Option::None,
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: msg.data.source_nft_contract_address.clone(),
                    code_hash,
                    msg: to_binary(&create_collection_msg)?,
                    funds: vec![],
                });

                let clog: Vec<Attribute> = vec![Claimed721EventInfo::new(
                    msg.data.lock_tx_chain,
                    msg.data.source_chain,
                    msg.data.transaction_hash,
                    msg.data.source_nft_contract_address,
                    msg.data.token_id,
                )
                .try_into()?];

                Ok(Response::new().add_message(message).add_attributes(clog))
            }
        }
    } else {
        return Err(StdError::generic_err("Invalid bridge state"));
    }?;
    Ok(res)
}

fn claim1155(deps: DepsMut, env: Env, info: MessageInfo, msg: ClaimMsg) -> StdResult<Response> {
    let balance = deps
        .querier
        .query_balance(env.contract.address.clone(), "uscrt".to_string())?
        .amount;
    let self_chain = config_read(deps.storage).load()?.self_chain;
    let _ = has_correct_fee(msg.data.fee.clone(), info);
    let type_erc_1155 = config_read(deps.storage).load()?.type_erc_1155;
    let validators_count = config_read(deps.storage).load()?.validators_count;
    let _ = matches_current_chain(deps.storage, msg.data.destination_chain.clone());

    if msg.data.nft_type != type_erc_1155 {
        return Err(StdError::generic_err("Invalid NFT type!"));
    }

    let hash = create_claim_data_hash(msg.data.clone());

    let exists = UNIQUE_IDENTIFIER_STORAGE.get(deps.storage, &hash);
    let _ = match exists {
        Some(_v) => {
            return Err(StdError::generic_err("Data already processed!"));
        }
        None => UNIQUE_IDENTIFIER_STORAGE.insert(deps.storage, &hash, &true),
    };

    let validators_to_reward = validate_signature(
        deps.api,
        hash,
        msg.signatures,
        validators_count.try_into().unwrap(),
    )?;

    let _ = reward_validators(
        deps.storage,
        msg.data.fee.clone(),
        validators_to_reward,
        balance.into(),
    );

    let duplicate_collection_address_option = ORIGINAL_TO_DUPLICATE_STORAGE.get(
        deps.storage,
        &(
            msg.data.source_nft_contract_address.clone(),
            msg.data.source_chain.clone(),
        ),
    );
    let mut duplicate_collection_address = OriginalToDuplicateContractInfo {
        chain: "".to_string(),
        contract_address: env.contract.address.clone(),
        code_hash: "".to_string(),
    };
    let mut has_duplicate: bool = false;
    let has_storage: bool;
    let storage_contract_option: Option<(Addr, String)>;
    let storage_contract: (Addr, String);

    // transfer_nft_msg;

    match duplicate_collection_address_option {
        Some(v) => {
            duplicate_collection_address = v;
            storage_contract_option = DUPLICATE_STORAGE_1155.get(
                deps.storage,
                &(
                    duplicate_collection_address.contract_address.to_string(),
                    self_chain,
                ),
            );
            has_duplicate = true
        }
        None => {
            storage_contract_option = ORIGINAL_STORAGE_1155.get(
                deps.storage,
                &(msg.data.source_nft_contract_address.clone(), self_chain),
            );
        }
    }

    match storage_contract_option {
        Some(v) => {
            storage_contract = v;
            has_storage = true;
        }
        None => {
            storage_contract = (Addr::unchecked("none"), "none".to_string());
            has_storage = false;
        }
    }
    // ===============================/ hasDuplicate && hasStorage /=======================
    let res = if has_duplicate && has_storage {

        let is_storage_is_nft_owner_option = NFT_COLLECTION_OWNER.get(
            deps.storage,
            &(
                duplicate_collection_address
                    .contract_address
                    .clone()
                    .into_string(),
                msg.data.token_id.to_string(),
            ),
        );

        match is_storage_is_nft_owner_option {
            Some(_v) => {
                if _v.1 >= msg.data.token_amount {
                    let create_unlock_msg = storage1155::msg::Storage1155ExecuteMsg::UnLockToken {
                        token_id: msg.data.token_id.clone(),
                        to: msg.data.destination_user_address.clone(),
                        amount: msg.data.token_amount.clone(),
                    };

                    let message = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                        contract_addr: storage_contract.0.clone().into_string(),
                        code_hash: storage_contract.1,
                        msg: to_binary(&create_unlock_msg)?,
                        funds: vec![],
                    });

                    let log: Vec<Attribute> = vec![
                        UnLock1155EventInfo::new(
                            msg.data.destination_user_address,
                            msg.data.token_id.clone(),
                            storage_contract.0.to_string(),
                            msg.data.token_amount,
                        )
                        .try_into()?,
                        Claimed1155EventInfo::new(
                            msg.data.lock_tx_chain,
                            msg.data.source_chain,
                            msg.data.transaction_hash,
                            duplicate_collection_address.contract_address.to_string(),
                            msg.data.token_id.clone(),
                            msg.data.token_amount,
                        )
                        .try_into()?,
                    ];

                    let _ = NFT_COLLECTION_OWNER.remove(
                        deps.storage,
                        &(
                            duplicate_collection_address.contract_address.into_string(),
                            msg.data.token_id,
                        ),
                    );

                    Ok(Response::new().add_message(message).add_attributes(log))
                } else {
                    let to_mint =
                    Uint256::from(msg.data.token_amount.clone()) - Uint256::from(_v.1);

                    let create_unlock_msg = storage1155::msg::Storage1155ExecuteMsg::UnLockToken {
                        token_id: msg.data.token_id.clone(),
                        to: msg.data.destination_user_address.clone(),
                        amount: _v.1,
                    };

                    let message_unlock = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                        contract_addr: storage_contract.0.clone().into_string(),
                        code_hash: storage_contract.1,
                        msg: to_binary(&create_unlock_msg)?,
                        funds: vec![],
                    });

                    let log: Vec<Attribute> = vec![
                        UnLock1155EventInfo::new(
                            msg.data.destination_user_address.clone(),
                            msg.data.token_id.clone(),
                            storage_contract.0.to_string(),
                            _v.1,
                        )
                        .try_into()?,
                        Claimed1155EventInfo::new(
                            msg.data.lock_tx_chain,
                            msg.data.source_chain,
                            msg.data.transaction_hash,
                            duplicate_collection_address.contract_address.to_string(),
                            msg.data.token_id.clone(),
                            msg.data.token_amount,
                        )
                        .try_into()?,
                    ];

                    let _ = NFT_COLLECTION_OWNER.remove(
                        deps.storage,
                        &(
                            duplicate_collection_address
                                .contract_address
                                .clone()
                                .into_string(),
                            msg.data.token_id.clone(),
                        ),
                    );

                    let create_collection_msg = Snip1155ExecuteMsg::MintTokens {
                        mint_tokens: vec![TokenAmount {
                            token_id: msg.data.token_id.clone(),
                            balances: vec![TokenIdBalance {
                                address: msg.data.destination_user_address.clone(),
                                amount: to_mint,
                            }],
                        }],
                        memo: Option::None,
                        padding: Option::None,
                    };

                    let message_mint = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                        contract_addr: duplicate_collection_address.contract_address.into_string(),
                        code_hash: duplicate_collection_address.code_hash,
                        msg: to_binary(&create_collection_msg)?,
                        funds: vec![],
                    });

                    Ok(Response::new()
                        .add_messages(vec![message_unlock, message_mint])
                        .add_attributes(log))
                }
            }
            None => {
                let create_new_token = Snip1155ExecuteMsg::CurateTokenIds {
                    initial_tokens: vec![CurateTokenId {
                        token_info: TokenInfoMsg {
                            token_id: msg.data.token_id.to_string(),
                            name: msg.data.name.clone(),
                            symbol: msg.data.symbol.clone(),
                            token_config: TknConfig::Fungible {
                                minters: vec![env.contract.address.clone()],
                                decimals: 6,
                                public_total_supply: true,
                                enable_mint: true,
                                enable_burn: true,
                                minter_may_update_metadata: true,
                            },
                            public_metadata: Some(Snip1155Meta {
                                token_uri: Some(msg.data.metadata.clone()),
                                extension: Option::None,
                            }),
                            private_metadata: Option::None,
                        },
                        balances: vec![TokenIdBalance {
                            address: msg.data.destination_user_address,
                            amount: Uint256::from(msg.data.token_amount),
                        }],
                    }],
                    memo: Option::None,
                    padding: Option::None,
                };

                let message = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                    contract_addr: duplicate_collection_address
                        .contract_address
                        .clone()
                        .into_string(),
                    code_hash: duplicate_collection_address.code_hash,
                    msg: to_binary(&create_new_token)?,
                    funds: vec![],
                });

                let emit: Vec<Attribute> = vec![Claimed1155EventInfo::new(
                    msg.data.lock_tx_chain,
                    msg.data.source_chain,
                    msg.data.transaction_hash,
                    duplicate_collection_address.contract_address.to_string(),
                    msg.data.token_id.clone(),
                    msg.data.token_amount,
                )
                .try_into()?];

                Ok(Response::new().add_message(message).add_attributes(emit))
            }
        }
    }
    // ===============================/ hasDuplicate && NOT hasStorage /=======================
    else if has_duplicate && !has_storage {

        let token_info_msg = Snip1155QueryMsg::Balance {
            owner: msg.data.destination_user_address.clone(),
            viewer: env.contract.address.clone(),
            key: "key".to_string(),
            token_id: msg.data.token_id.to_string(),
        };

        let token_info_query_result = token_info_msg.query::<Empty, Balance>(
            deps.querier,
            duplicate_collection_address.code_hash.clone(),
            duplicate_collection_address.contract_address.clone().to_string(),
        );

        let mut token_exists = false;
        let message;

        match token_info_query_result {
            Ok(result) => {
                deps.api.debug(
                    format!(
                        "OKOKOK",
                    )
                    .as_str(),
                );
                        deps.api.debug(
                            format!(
                                "msg.data.token_id {}",
                                msg.data.token_id
                            )
                            .as_str(),
                        );
                        if result.amount >= 0u128.into() {
                            token_exists = true;
                        }
            },
            Err(e) => {
                deps.api.debug(
                    format!(
                        "ERRRRR {}",e.to_string()
                    )
                    .as_str(),
                );
                token_exists = false;
            },
        };

        if token_exists {
            let mint_token = Snip1155ExecuteMsg::MintTokens {
                mint_tokens: vec![TokenAmount {
                    token_id: msg.data.token_id.to_string(),
                    balances: vec![TokenIdBalance {
                        address: msg.data.destination_user_address.clone(),
                        amount: msg.data.token_amount.clone().into(),
                    }],
                }],
                memo: Option::None,
                padding: Option::None,
            };

            message = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                contract_addr: duplicate_collection_address
                    .contract_address
                    .clone()
                    .into_string(),
                code_hash: duplicate_collection_address.code_hash,
                msg: to_binary(&mint_token)?,
                funds: vec![],
            });
        }
        else {
            let create_new_token = Snip1155ExecuteMsg::CurateTokenIds {
                initial_tokens: vec![CurateTokenId {
                    token_info: TokenInfoMsg {
                        token_id: msg.data.token_id.to_string(),
                        name: msg.data.name.clone(),
                        symbol: msg.data.symbol.clone(),
                        token_config: TknConfig::Fungible {
                            minters: vec![env.contract.address.clone()],
                            decimals: 6,
                            public_total_supply: true,
                            enable_mint: true,
                            enable_burn: true,
                            minter_may_update_metadata: true,
                        },
                        public_metadata: Some(Snip1155Meta {
                            token_uri: Some(msg.data.metadata.clone()),
                            extension: Option::None,
                        }),
                        private_metadata: Option::None,
                    },
                    balances: vec![TokenIdBalance {
                        address: msg.data.destination_user_address,
                        amount: Uint256::from(msg.data.token_amount),
                    }],
                }],
                memo: Option::None,
                padding: Option::None,
            };

            message = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                contract_addr: duplicate_collection_address
                    .contract_address
                    .clone()
                    .into_string(),
                code_hash: duplicate_collection_address.code_hash,
                msg: to_binary(&create_new_token)?,
                funds: vec![],
            });
        }

        let emit: Vec<Attribute> = vec![Claimed1155EventInfo::new(
            msg.data.lock_tx_chain,
            msg.data.source_chain,
            msg.data.transaction_hash,
            duplicate_collection_address.contract_address.to_string(),
            msg.data.token_id.clone(),
            msg.data.token_amount,
        )
        .try_into()?];

        Ok(Response::new().add_message(message).add_attributes(emit))
    }
    // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
    else if !has_duplicate && !has_storage {
        // new collection
        deploy_collection_1155(
            deps,
            msg.data.name,
            msg.data.symbol,
            env.contract.address,
            msg.data.source_nft_contract_address,
            msg.data.source_chain.clone(),
            msg.data.destination_user_address,
            msg.data.token_id,
            msg.data.token_amount,
            msg.data.royalty,
            msg.data.royalty_receiver,
            msg.data.metadata,
            msg.data.transaction_hash,
            msg.data.lock_tx_chain,
            env.block.time.seconds(),
        )
    }
    // ===============================/ NOT hasDuplicate && hasStorage /=======================
    else if !has_duplicate && has_storage {

        deps.api.debug(
            format!(
                "not duplicate and has storage",
            )
            .as_str(),
        );

        let get_owner_of = Snip1155QueryMsg::Balance {
            owner: storage_contract.0.clone(),
            viewer: env.contract.address,
            key: "key".to_string(),
            token_id: msg.data.token_id.to_string(),
        };
        let code_hash = CODEHASHES
            .get(
                deps.storage,
                &deps
                    .api
                    .addr_validate(&msg.data.source_nft_contract_address.clone())?,
            )
            .unwrap();

        let original_collection = get_owner_of.query::<Empty, Balance>(
            deps.querier,
            code_hash.clone(),
            msg.data.source_nft_contract_address.clone(),
        )?;

        deps.api.debug(
            format!(
                "not duplicate and has storage 13123123123123",
            )
            .as_str(),
        );

        if original_collection.amount >= msg.data.token_amount.into() {
            let create_unlock_msg = storage1155::msg::Storage1155ExecuteMsg::UnLockToken {
                token_id: msg.data.token_id.clone(),
                amount: msg.data.token_amount,
                to: msg.data.destination_user_address.clone(),
            };

            let message = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: storage_contract.0.clone().into_string(),
                code_hash: storage_contract.1,
                msg: to_binary(&create_unlock_msg)?,
                funds: vec![],
            });

            let log: Vec<Attribute> = vec![
                UnLock1155EventInfo::new(
                    msg.data.destination_user_address.clone(),
                    msg.data.token_id.clone(),
                    storage_contract.0.to_string(),
                    msg.data.token_amount,
                )
                .try_into()?,
                Claimed1155EventInfo::new(
                    msg.data.lock_tx_chain,
                    msg.data.source_chain,
                    msg.data.transaction_hash,
                    msg.data.source_nft_contract_address.clone(),
                    msg.data.token_id.clone(),
                    msg.data.token_amount,
                )
                .try_into()?,
            ];

            let _ = NFT_COLLECTION_OWNER.remove(
                deps.storage,
                &(
                    msg.data.source_nft_contract_address.clone(),
                    msg.data.token_id.to_string(),
                ),
            );

            Ok(Response::new().add_message(message).add_attributes(log))
        } else {
            // CANT BE THERE
            let to_mint = Uint256::from(msg.data.token_amount.clone()) - Uint256::from(original_collection.amount);

            let create_unlock_msg = storage1155::msg::Storage1155ExecuteMsg::UnLockToken {
                token_id: msg.data.token_id.clone(),
                amount: original_collection.amount.into(),
                to: msg.data.destination_user_address.clone(),
            };

            let message_unlock = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: storage_contract.0.clone().into_string(),
                code_hash: storage_contract.1,
                msg: to_binary(&create_unlock_msg)?,
                funds: vec![],
            });

            let log: Vec<Attribute> = vec![
                UnLock1155EventInfo::new(
                    msg.data.destination_user_address.clone(),
                    msg.data.token_id.clone(),
                    storage_contract.0.to_string(),
                    original_collection.amount.into(),
                )
                .try_into()?,
                Claimed1155EventInfo::new(
                    msg.data.lock_tx_chain,
                    msg.data.source_chain,
                    msg.data.transaction_hash,
                    msg.data.source_nft_contract_address.clone(),
                    msg.data.token_id.clone(),
                    msg.data.token_amount,
                )
                .try_into()?,
            ];

            let _ = NFT_COLLECTION_OWNER.remove(
                deps.storage,
                &(
                    msg.data.source_nft_contract_address.clone(),
                    msg.data.token_id.to_string(),
                ),
            );

            let create_collection_msg = Snip1155ExecuteMsg::MintTokens {
                mint_tokens: vec![TokenAmount {
                    token_id: msg.data.token_id.to_string(),
                    balances: vec![TokenIdBalance {
                        address: msg.data.destination_user_address.clone(),
                        amount: to_mint,
                    }],
                }],
                memo: Option::None,
                padding: Option::None,
            };

            let message_mint = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                contract_addr: msg.data.source_nft_contract_address,
                code_hash,
                msg: to_binary(&create_collection_msg)?,
                funds: vec![],
            });

            Ok(Response::new()
                .add_messages(vec![message_unlock, message_mint])
                .add_attributes(log))
        }
    } else {
        return Err(StdError::generic_err("Invalid bridge state"));
    }?;

    Ok(res)
}
// Queries
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: BridgeQueryMsg) -> StdResult<Binary> {
    match msg {
        BridgeQueryMsg::GetValidatorsCount {} => to_binary(&validators_count(deps)?),
        BridgeQueryMsg::GetValidator { address } => to_binary(&validators(deps, address)?),
        BridgeQueryMsg::GetCollectionDeployer {} => to_binary(&collection_deployer(deps)?),
        BridgeQueryMsg::GetStorageDeployer {} => to_binary(&storage_deployer(deps)?),
        BridgeQueryMsg::GetOriginalStorage721 {
            contract_address,
            chain,
        } => to_binary(&original_storage721(deps, contract_address, chain)?),
        BridgeQueryMsg::GetDuplicateStorage721 {
            contract_address,
            chain,
        } => to_binary(&duplicate_storage721(deps, contract_address, chain)?),
        BridgeQueryMsg::GetOriginalToDuplicate {
            contract_address,
            chain,
        } => to_binary(&original_to_duplicate(deps, contract_address, chain)?),
        BridgeQueryMsg::GetDuplicateToOriginal {
            contract_address,
            chain,
        } => to_binary(&duplicate_to_original(deps, contract_address, chain)?),
    }
}

fn validators_count(deps: Deps) -> StdResult<BridgeQueryAnswer> {
    let state = config_read(deps.storage).load()?;
    Ok(BridgeQueryAnswer::ValidatorCountResponse {
        count: state.validators_count,
    })
}

fn validators(deps: Deps, address: Binary) -> StdResult<BridgeQueryAnswer> {
    let validator_option = VALIDATORS_STORAGE.get(deps.storage, &address);
    Ok(BridgeQueryAnswer::Validator {
        data: validator_option,
    })
}

fn collection_deployer(deps: Deps) -> StdResult<BridgeQueryAnswer> {
    let collection_deployer = config_read(deps.storage).load()?.collection_deployer;
    Ok(BridgeQueryAnswer::CollectionDeployer {
        data: collection_deployer,
    })
}

fn storage_deployer(deps: Deps) -> StdResult<BridgeQueryAnswer> {
    let storage_deployer = config_read(deps.storage).load()?.storage_deployer;
    Ok(BridgeQueryAnswer::StorageDeployer {
        data: storage_deployer,
    })
}

fn original_storage721(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<BridgeQueryAnswer> {
    let storage_option = ORIGINAL_STORAGE_721.get(deps.storage, &(contract_address, chain));
    Ok(BridgeQueryAnswer::Storage {
        data: storage_option,
    })
}

fn duplicate_storage721(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<BridgeQueryAnswer> {
    let storage_option = DUPLICATE_STORAGE_721.get(deps.storage, &(contract_address, chain));
    Ok(BridgeQueryAnswer::Storage {
        data: storage_option,
    })
}

fn original_to_duplicate(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<BridgeQueryAnswer> {
    let storage_option =
        ORIGINAL_TO_DUPLICATE_STORAGE.get(deps.storage, &(contract_address, chain));
    Ok(BridgeQueryAnswer::OriginalToDuplicate {
        data: storage_option,
    })
}

fn duplicate_to_original(
    deps: Deps,
    contract_address: Addr,
    chain: String,
) -> StdResult<BridgeQueryAnswer> {
    let storage_option =
        DUPLICATE_TO_ORIGINAL_STORAGE.get(deps.storage, &(contract_address, chain));
    Ok(BridgeQueryAnswer::DuplicateToOriginal {
        data: storage_option,
    })
}

// Replies
#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        STORAGE_DEPLOYER_REPLY_ID => handle_storage_deployer_reply(_deps, msg),
        STORAGE_DEPLOYER_721_REPLY_ID => handle_storage_reply_721(_deps, msg),
        STORAGE_DEPLOYER_1155_REPLY_ID => {
            handle_storage_reply_1155(_deps, msg)
        }
        COLLECTION_DEPLOYER_REPLY_ID => handle_collection_deployer_reply(_deps, msg),
        COLLECTION_DEPLOYER_721_REPLY_ID => handle_collection_reply_721(_deps, msg),
        COLLECTION_DEPLOYER_1155_REPLY_ID => handle_collection_reply_1155(_deps, msg),
        id => Err(ContractError::UnexpectedReplyId { id }),
    }
}

fn handle_storage_deployer_reply(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyStorageDeployerInfo = from_binary(&bin)?;
                register_storage_deployer_impl(_deps, reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with storage deployer".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

fn handle_storage_reply_721(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyStorage721Info = from_binary(&bin)?;
                register_storage_721_impl(_deps, reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with storage address 721".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

fn handle_storage_reply_1155(
    _deps: DepsMut,
    msg: Reply,
) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyStorage1155Info = from_binary(&bin)?;
                register_storage_1155_impl(_deps, reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with storage address 1155".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

fn handle_collection_deployer_reply(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyCollectionDeployerInfo = from_binary(&bin)?;
                register_collection_deployer_impl(_deps, reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with collection deployer".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

fn handle_collection_reply_721(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyCollection721Info = from_binary(&bin)?;
                register_collection_721_impl(_deps, reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with collection address 721".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

fn handle_collection_reply_1155(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyCollection1155Info = from_binary(&bin)?;
                register_collection_1155_impl(_deps, reply_info)
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with collection address 1155".to_string(),
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
fn register_storage_deployer_impl(
    deps: DepsMut,
    reply_info: ReplyStorageDeployerInfo,
) -> Result<Response, ContractError> {
    config(deps.storage).update(|mut state| -> Result<_, StdError> {
        state.storage_deployer = reply_info.address.clone();
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("storage_deployer", &reply_info.address))
}

fn register_storage_721_impl(
    deps: DepsMut,
    reply_info: ReplyStorage721Info,
) -> Result<Response, ContractError> {
    let self_chain = config(deps.storage).load()?.self_chain;
    if reply_info.is_original {
        let _ = ORIGINAL_STORAGE_721.insert(
            deps.storage,
            &(reply_info.source_nft_contract_address.to_string().clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    } else {
        let _ = DUPLICATE_STORAGE_721.insert(
            deps.storage,
            &(reply_info.source_nft_contract_address.to_string().clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    }
    let res = transfer_to_storage_721(
        deps.storage,
        reply_info.address.clone(),
        deps.api.addr_validate(&reply_info.source_nft_contract_address.clone().into_string())?,
        reply_info.token_id,
        reply_info.collection_code_hash,
    )?;
    Ok(res.add_attribute("storage_address_721", &reply_info.address))
}

fn register_storage_1155_impl(
    deps: DepsMut,
    reply_info: ReplyStorage1155Info,
) -> Result<Response, ContractError> {

    deps.api
        .debug(format!("register_storage_1155_impl").as_str());

    let self_chain = config(deps.storage).load()?.self_chain;
    if reply_info.is_original {
        let _ = ORIGINAL_STORAGE_1155.insert(
            deps.storage,
            &(reply_info.source_nft_contract_address.to_string().clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    } else {
        let _ = DUPLICATE_STORAGE_1155.insert(
            deps.storage,
            &(reply_info.source_nft_contract_address.to_string().clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    }
    deps.api
        .debug(format!("transfer_to_storage_1155").as_str());

    let res = transfer_to_storage_1155(
        deps.storage,
        deps.api,
        reply_info.from,
        reply_info.address.clone(),
        deps.api.addr_validate(&reply_info.source_nft_contract_address.clone().into_string())?,
        reply_info.token_id,
        reply_info.token_amount,
        reply_info.collection_code_hash,
    )?;

    deps.api
        .debug(format!("transfer_to_storage_115522222222222").as_str());

    Ok(res.add_attribute("storage_address_1155", &reply_info.address))
}

fn register_collection_deployer_impl(
    deps: DepsMut,
    reply_info: ReplyCollectionDeployerInfo,
) -> Result<Response, ContractError> {
    config(deps.storage).update(|mut state| -> Result<_, StdError> {
        state.collection_deployer = reply_info.address.clone();
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("collection_deployer", &reply_info.address))
}

fn register_collection_721_impl(
    deps: DepsMut,
    reply_info: ReplyCollection721Info,
) -> Result<Response, ContractError> {
    let self_chain = config(deps.storage).load()?.self_chain;

    let _ = ORIGINAL_TO_DUPLICATE_STORAGE.insert(
        deps.storage,
        &(
            reply_info.source_nft_contract_address.clone(),
            reply_info.source_chain.clone(),
        ),
        &OriginalToDuplicateContractInfo {
            chain: self_chain.clone(),
            contract_address: reply_info.address.clone(),
            code_hash: reply_info.code_hash.clone(),
        },
    );

    let _ = DUPLICATE_TO_ORIGINAL_STORAGE.insert(
        deps.storage,
        &(reply_info.address.clone(), self_chain),
        &DuplicateToOriginalContractInfo {
            chain: reply_info.source_chain.clone(),
            contract_address: reply_info.source_nft_contract_address,
            code_hash: "".to_string(),
        },
    );

    let create_collection_msg = snip721::msg::Snip721ExecuteMsg::MintNft {
        token_id: Some(reply_info.token_id.to_string()),
        owner: Some(reply_info.destination_user_address.into_string()),
        public_metadata: Some(Snip721Meta {
            token_uri: Some(reply_info.metadata),
            extension: Option::None,
        }),
        private_metadata: Option::None,
        serial_number: Option::None,
        royalty_info: Some(RoyaltyInfo {
            decimal_places_in_rates: 4,
            royalties: vec![Royalty {
                rate: reply_info.royalty,
                recipient: reply_info.royalty_receiver.into_string(),
            }],
        }),
        transferable: Some(true),
        memo: Option::None,
        padding: Option::None,
    };

    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: reply_info.address.clone().into_string(),
        code_hash: reply_info.code_hash,
        msg: to_binary(&create_collection_msg)?,
        funds: vec![],
    });
    let emit: Vec<Attribute> = vec![Claimed721EventInfo::new(
        reply_info.lock_tx_chain,
        reply_info.source_chain,
        reply_info.transaction_hash,
        reply_info.address.to_string(),
        reply_info.token_id,
    )
    .try_into()?];

    Ok(Response::new()
        .add_message(message)
        .add_attributes(emit)
        .add_attribute("collection_address_721", &reply_info.address))
}

fn register_collection_1155_impl(
    deps: DepsMut,
    reply_info: ReplyCollection1155Info,
) -> Result<Response, ContractError> {
    let self_chain = config(deps.storage).load()?.self_chain;

    let _ = ORIGINAL_TO_DUPLICATE_STORAGE.insert(
        deps.storage,
        &(
            reply_info.source_nft_contract_address.clone(),
            reply_info.source_chain.clone(),
        ),
        &OriginalToDuplicateContractInfo {
            chain: self_chain.clone(),
            contract_address: reply_info.address.clone(),
            code_hash: reply_info.code_hash.clone(),
        },
    );

    let _ = DUPLICATE_TO_ORIGINAL_STORAGE.insert(
        deps.storage,
        &(reply_info.address.clone(), self_chain),
        &DuplicateToOriginalContractInfo {
            chain: reply_info.source_chain.clone(),
            contract_address: reply_info.source_nft_contract_address,
            code_hash: "".to_string(),
        },
    );

    // let create_collection_msg = Snip1155ExecuteMsg::MintTokens {
    //     mint_tokens: vec![TokenAmount {
    //         token_id: reply_info.token_id.to_string(),
    //         balances: vec![TokenIdBalance {
    //             address: reply_info.destination_user_address.clone(),
    //             amount: reply_info.token_amount.into(),
    //         }],
    //     }],
    //     memo: Option::None,
    //     padding: Option::None,
    // };

    // let message = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: reply_info.address.clone().into_string(),
    //     code_hash: reply_info.code_hash,
    //     msg: to_binary(&create_collection_msg)?,
    //     funds: vec![],
    // });

    let emit: Vec<Attribute> = vec![Claimed1155EventInfo::new(
        reply_info.lock_tx_chain,
        reply_info.source_chain,
        reply_info.transaction_hash,
        reply_info.address.to_string(),
        reply_info.token_id,
        reply_info.token_amount,
    )
    .try_into()?];

    Ok(Response::new()
        // .add_message(message)
        .add_attributes(emit)
        .add_attribute("collection_address_1155", &reply_info.address))
}
