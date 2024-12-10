use std::collections::BTreeMap;

use collection_deployer::msg::{CollectionDeployerExecuteMsg, CollectionDeployerInstantiateMsg};
use cosm_nft::royalty::RoyaltyData;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Addr, Api, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, StdResult, Storage, SubMsg, SubMsgResult, Uint128, WasmMsg
};

use cw0::{parse_execute_response_data, parse_reply_instantiate_data};
use cw_storage_plus::Map;

use crate::error::ContractError;
use crate::events::{
    AddNewValidatorEventInfo, BlacklistValidatorEventInfo, ClaimedEventInfo, LockedEventInfo, RewardValidatorEventInfo, UnLock721EventInfo
};
use crate::msg::{
    BridgeExecuteMsg, BridgeQueryMsg, GetCollectionDeployerResponse,
    GetDuplicateToOriginalResponse, GetOriginalToDuplicateResponse, GetStorageDeployerResponse,
    GetStorageResponse, GetValidatorCountResponse, GetValidatorResponse,
};

use crate::state::{
    BLACKLISTED_VALIDATORS, COLLECTION_DEPLOYER_721_REPLY_ID, COLLECTION_DEPLOYER_REPLY_ID,
    COLLETION_DEPLOYER_CODE, CONFIG, DUPLICATE_STORAGE_721, DUPLICATE_TO_ORIGINAL_STORAGE,
    NFT_COLLECTION_OWNER, ORIGINAL_STORAGE_721, ORIGINAL_TO_DUPLICATE_STORAGE,
    STORAGE_DEPLOYER_721_REPLY_ID, STORAGE_DEPLOYER_CODE, STORAGE_DEPLOYER_REPLY_ID,
    UNIQUE_IDENTIFIER_STORAGE, VALIDATORS_STORAGE,
};

use crate::structs::{
    AddValidatorMsg, BlacklistValidatorMsg, BridgeInstantiateMsg, ClaimData, ClaimMsg,
    ClaimValidatorRewardsMsg, DuplicateToOriginalContractInfo, Lock721Msg,
    OriginalToDuplicateContractInfo, ReplyCollectionDeployerInfo, ReplyCollectionInfo,
    ReplyStorageDeployerInfo, ReplyStorageInfo, SignerAndSignature, State, Validator, VerifyMsg,
};
use cosm_nft::NftExecuteMsg;
use nft_store::msg::NftStoreExecuteMsg;
use sha2::{Digest, Sha256};
use store_deployer::msg::{StoreFactoryExecuteMsg, StoreFactoryInstantiateMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: BridgeInstantiateMsg,
) -> StdResult<Response> {
    let mut validators_count = 0;

    for (pubk, address) in msg.validators {
        validators_count += 1;
        VALIDATORS_STORAGE.save(
            deps.storage,
            pubk.0,
            &Validator {
                address,
                added: true,
                pending_reward: 0,
            },
        )?;
    }

    let state = State {
        collection_deployer: env.contract.address.clone(),
        storage_deployer: env.contract.address,
        validators_count,
        self_chain: msg.chain_type,
        type_erc_721: "singular".to_owned(),
        type_erc_1155: "multiple".to_owned(),
    };

    CONFIG.save(deps.storage, &state)?;

    STORAGE_DEPLOYER_CODE.save(deps.storage, &msg.storage_deployer_code_id)?;
    COLLETION_DEPLOYER_CODE.save(deps.storage, &msg.collection_deployer_code_id)?;

    let init_storage_deployer_msg = StoreFactoryInstantiateMsg {
        storage721_code_id: msg.storage721_code_id,
    };

    let init_storage_deployer_sub_msg = SubMsg::reply_always(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: msg.storage_deployer_code_id,
            msg: to_json_binary(&init_storage_deployer_msg)?,
            funds: vec![],
            label: msg.storage_label,
        }),
        STORAGE_DEPLOYER_REPLY_ID,
    );

    let init_collection_deployer_msg = CollectionDeployerInstantiateMsg {
        collection721_code_id: msg.collection721_code_id,
    };

    let init_collection_deployer_submsg = SubMsg::reply_always(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: msg.collection_deployer_code_id,
            msg: to_json_binary(&init_collection_deployer_msg)?,
            funds: vec![],
            label: msg.collection_label,
        }),
        COLLECTION_DEPLOYER_REPLY_ID,
    );
    Ok(Response::new().add_submessages(vec![
        init_storage_deployer_sub_msg,
        init_collection_deployer_submsg,
    ]))
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
        BridgeExecuteMsg::Claim721 { data } => claim721(deps, env, info, data),
        BridgeExecuteMsg::VerifySig { data } => verify_sig(deps, data),
    }
}

fn verify_sig(deps: DepsMut, msg: VerifyMsg) -> StdResult<Response> {
    let serialized = serde_json::to_vec(&msg.claim_data_as_binary).unwrap();
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
    if destination_chain == CONFIG.load(storage)?.self_chain {
        return Err(StdError::generic_err("Invalid destination chain!"));
    } else {
        Ok(Response::default())
    }
}

fn has_correct_fee(fee: u128, info: MessageInfo) -> StdResult<Response> {
    let msg_value = info.funds[0].amount;

    if Uint128::from(fee) <= msg_value {
        Ok(Response::default())
    } else {
        return Err(StdError::generic_err("data.fee LESS THAN sent amount!"));
    }
}

fn blacklist_validator(deps: DepsMut, blacklist_msg: BlacklistValidatorMsg) -> StdResult<Response> {
    if blacklist_msg.signatures.len() <= 0 {
        return Err(StdError::generic_err("Must have signatures!"));
    }
    if !VALIDATORS_STORAGE.has( deps.storage, blacklist_msg.validator.0 .0.clone()) {
        return Err(StdError::generic_err("Validator is not added"));
    }
    let percentage = validate_signatures(
        deps.api,
        &blacklist_msg.validator.0.clone(),
        &blacklist_msg.signatures,
    )?;
    let state = CONFIG.load(deps.storage)?;
    if percentage < required_threshold(state.validators_count as u128) {
        return Err(StdError::generic_err("Threshold not reached!"));
    }
    VALIDATORS_STORAGE.remove(deps.storage, blacklist_msg.validator.0 .0.clone());
    CONFIG.update(deps.storage, |mut state| -> Result<_, StdError> {
        state.validators_count -= 1;
        Ok(state) // Return the modified state
    })?;
    BLACKLISTED_VALIDATORS.save(deps.storage, blacklist_msg.validator.0 .0, &true)?;
    let log: Vec<Attribute> = vec![BlacklistValidatorEventInfo::new(blacklist_msg.validator.1).try_into()?];
    Ok(Response::new().add_attributes(log))
}

fn add_validator(deps: DepsMut, add_validator_msg: AddValidatorMsg) -> StdResult<Response> {
    if BLACKLISTED_VALIDATORS.has(deps.storage, add_validator_msg.validator.0 .0.clone()) {
        return Err(StdError::generic_err("validator blacklisted"));
    }

    if add_validator_msg.signatures.is_empty() {
        return Err(StdError::generic_err("Must have signatures!"));
    }

    let state = CONFIG.load(deps.storage)?;
    if VALIDATORS_STORAGE.has(deps.storage, add_validator_msg.validator.0 .0.clone()) {
        return Err(StdError::generic_err("Validator already added"));
    }

    let percentage = validate_signatures(
        deps.api,
        &add_validator_msg.validator.0.clone(),
        &add_validator_msg.signatures,
    )?;
    if percentage < required_threshold(state.validators_count as u128) {
        return Err(StdError::generic_err("Threshold not reached!"));
    }

    Ok(add_validator_to_state(
        deps.storage,
        &add_validator_msg.validator,
    )?)
}

fn validate_signatures(
    api: &dyn Api,
    key: &Binary,
    sigs: &Vec<SignerAndSignature>,
) -> StdResult<i128> {
    let mut percentage = 0;
    let serialized = key.to_string();
    let mut hasher = Sha256::new();

    hasher.update(serialized);
    let hash: [u8; 32] = hasher.finalize().into();
    let mut unique = BTreeMap::new();

    for ele in sigs {
        if unique.contains_key(&ele.signer_address) {
            continue;
        }
        unique.insert(ele.signer_address.clone(), true);
        if verify_signature(api, &ele.signature, &ele.signer_address, &hash)? {
            percentage += 1;
        }
    }
    Ok(percentage)
}

fn verify_signature(
    api: &dyn Api,
    signature: &[u8],
    signer_address: &[u8],
    hash: &[u8; 32],
) -> StdResult<bool> {
    if signature.len() == 64 && signer_address.len() == 33 {
        return Ok(api.secp256k1_verify(hash, &signature, &signer_address)?);
    }
    Ok(false)
}

fn required_threshold(validators_count: u128) -> i128 {
    ((validators_count * 2) / 3) as i128 + 1
}

fn add_validator_to_state(
    storage: &mut dyn Storage,
    validator: &(Binary, Addr),
) -> StdResult<Response> {
    VALIDATORS_STORAGE.save(
        storage,
        validator.0 .0.clone(),
        &Validator {
            address: validator.1.clone(),
            added: true,
            pending_reward: 0,
        },
    )?;
    CONFIG.update(storage, |mut state| -> Result<_, StdError> {
        state.validators_count += 1;
        Ok(state) // Return the modified state
    })?;

    let log: Vec<Attribute> = vec![AddNewValidatorEventInfo::new(validator.1.clone()).try_into()?];

    Ok(Response::new().add_attributes(log)) // Indicate successful completion of the function
}

fn claim_validator_rewards(deps: DepsMut, data: ClaimValidatorRewardsMsg) -> StdResult<Response> {
    let rewards_option = VALIDATORS_STORAGE.may_load(deps.storage, data.validator.clone().0)?;
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

            let _ = VALIDATORS_STORAGE.save(deps.storage, data.validator.0.clone(), &v);

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
    storage_mapping_721: &Map<'static, (String, String), Addr>,
    source_nft_contract_address: Addr,
    token_id: String,
    collection_code_id: u64,
    owner: Addr,
    is_original: bool,
) -> StdResult<Response> {
    let storage_address_option = storage_mapping_721.may_load(
        deps.storage,
        (
            source_nft_contract_address.clone().into_string(),
            self_chain.clone(),
        ),
    )?;

    match storage_address_option {
        Some(v) => transfer_to_storage_721(
            deps.storage,
            v,
            source_nft_contract_address.clone(),
            token_id,
        ),
        None => {
            let create_storage_msg = StoreFactoryExecuteMsg::CreateStorage721 {
                label: source_nft_contract_address.clone().into_string(),
                collection_address: source_nft_contract_address.clone(),
                collection_code_id,
                owner: owner.into_string(),
                is_original,
                token_id,
            };

            let init_submsg = SubMsg::reply_always(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: CONFIG.load(deps.storage)?.storage_deployer.to_string(),
                    msg: to_json_binary(&create_storage_msg)?,
                    funds: vec![],
                }),
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
) -> StdResult<Response> {
    let transfer_msg = NftExecuteMsg::TransferNft {
        recipient: storage_address.clone().into_string(),
        token_id: token_id.to_string(),
    };

    let transfer_submsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: source_nft_contract_address.clone().into_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });

    let _ = NFT_COLLECTION_OWNER.save(
        storage,
        (
            source_nft_contract_address.clone().into_string(),
            token_id.to_string(),
        ),
        &(storage_address, 1u128),
    );

    Ok(Response::new().add_message(transfer_submsg))
}

fn lock721(deps: DepsMut, env: Env, msg: Lock721Msg) -> StdResult<Response> {
    let addr_result = deps
        .api
        .addr_validate(&msg.source_nft_contract_address.clone());
    let self_chain = CONFIG.load(deps.storage)?.self_chain.clone();
    match addr_result {
        Ok(_v) => {}
        Err(_) => {
            return Err(StdError::generic_err(
                "sourceNftContractAddress cannot be zero address",
            ));
        }
    }

    let original_collection_address_option = DUPLICATE_TO_ORIGINAL_STORAGE.may_load(
        deps.storage,
        (
            Addr::unchecked(msg.source_nft_contract_address.clone()),
            self_chain.clone(),
        ),
    )?;

    match original_collection_address_option {
        Some(_v) => {
            // notOriginal
            let log: Vec<Attribute> = vec![LockedEventInfo::new(
                msg.token_id.clone(),
                msg.destination_chain,
                msg.destination_user_address,
                _v.contract_address,
                1,
                CONFIG.load(deps.storage)?.type_erc_721,
                _v.chain,
            )
            .try_into()?];

            let res = check_storage_721(
                deps,
                self_chain.clone(),
                &DUPLICATE_STORAGE_721,
                Addr::unchecked(msg.source_nft_contract_address.clone()),
                msg.token_id,
                msg.collection_code_id,
                env.contract.address,
                false,
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
                CONFIG.load(deps.storage)?.type_erc_721,
                CONFIG.load(deps.storage)?.self_chain,
            )
            .try_into()?];

            let res = check_storage_721(
                deps,
                self_chain,
                &ORIGINAL_STORAGE_721,
                Addr::unchecked(msg.source_nft_contract_address.clone()),
                msg.token_id,
                msg.collection_code_id,
                env.contract.address,
                true,
            )?;

            Ok(res.add_attributes(log))
        }
    }
}

fn create_claim_data_hash(data: ClaimData) -> [u8; 32] {
    let serialized = data.concat_all_fields();
    let mut hasher = Sha256::new();
    hasher.update(serialized);
    let output = hasher.finalize().into();
    output
}

fn verify_signatures(
    api: &dyn Api,
    signature: &[u8],
    signer_address: &[u8],
    hash: &[u8; 32],
) -> StdResult<bool> {
    if signature.len() == 64 && signer_address.len() == 33 {
        return Ok(api.secp256k1_verify(hash, &signature, &signer_address)?);
    }
    Ok(false)
}

fn validate_signature(
    api: &dyn Api,
    hash: [u8; 32],
    signatures: Vec<SignerAndSignature>,
    validators_count: u128,
) -> StdResult<Vec<Binary>> {
    let mut percentage = 0;
    let mut arr: Vec<Binary> = Vec::new();
    for ele in signatures {
        if verify_signatures(api, &ele.signature, &ele.signer_address, &hash)? {
            percentage += 1;
            arr.push(ele.signer_address);
        }
    }

    if percentage < required_threshold(validators_count) {
        return Err(StdError::generic_err("Threshold not reached!"));
    }
    Ok(arr)
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

    if balance < fee {
        return Err(StdError::generic_err("No rewards available"));
    }

    let fee_per_validator = fee / validators_to_reward.len() as u128;

    for val in validators_to_reward {
        let validator_option = VALIDATORS_STORAGE.may_load(storage, val.0.clone())?;
        match validator_option {
            Some(mut v) => {
                v.pending_reward = v.pending_reward + fee_per_validator;

                let _ = VALIDATORS_STORAGE.save(storage, val.0, &v);
            }
            None => todo!(),
        }
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
) -> StdResult<Response> {
    let create_collection_msg = CollectionDeployerExecuteMsg::CreateCollection721 {
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

    let init_wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: CONFIG.load(deps.storage)?.collection_deployer.to_string(),
        msg: to_json_binary(&create_collection_msg)?,
        funds: vec![],
    });

    let init_submsg = SubMsg::reply_always(init_wasm_msg, COLLECTION_DEPLOYER_721_REPLY_ID);
    Ok(Response::new().add_submessage(init_submsg))
}

fn claim721(deps: DepsMut, env: Env, info: MessageInfo, msg: ClaimMsg) -> StdResult<Response> {
    let balance = deps
        .querier
        .query_balance(env.contract.address.clone(), "uscrt".to_string())?
        .amount;

    let self_chain = CONFIG.load(deps.storage)?.self_chain;

    let _ = has_correct_fee(msg.data.fee.clone(), info);

    let type_erc_721 = CONFIG.load(deps.storage)?.type_erc_721;

    let validators_count = CONFIG.load(deps.storage)?.validators_count;

    let _ = matches_current_chain(deps.storage, msg.data.destination_chain.clone());

    if msg.data.nft_type != type_erc_721 {
        return Err(StdError::generic_err("Invalid NFT type!"));
    }

    let hash = create_claim_data_hash(msg.data.clone());

    let exists = UNIQUE_IDENTIFIER_STORAGE.may_load(deps.storage, hash)?;
    let _ = match exists {
        Some(_v) => {
            return Err(StdError::generic_err("Data already processed!"));
        }
        None => UNIQUE_IDENTIFIER_STORAGE.save(deps.storage, hash, &true),
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

    let duplicate_collection_address_option = ORIGINAL_TO_DUPLICATE_STORAGE.may_load(
        deps.storage,
        (
            msg.data.source_nft_contract_address.clone(),
            msg.data.source_chain.clone(),
        ),
    )?;
    let mut duplicate_collection_address = OriginalToDuplicateContractInfo {
        chain: "".to_string(),
        contract_address: env.contract.address.clone(),
    };
    let mut has_duplicate: bool = false;
    let has_storage: bool;
    let storage_contract_option: Option<Addr>;
    let storage_contract: Addr;

    match duplicate_collection_address_option {
        Some(v) => {
            duplicate_collection_address = v;
            storage_contract_option = DUPLICATE_STORAGE_721.may_load(
                deps.storage,
                (
                    duplicate_collection_address.contract_address.to_string(),
                    self_chain,
                ),
            )?;
            has_duplicate = true
        }
        None => {
            storage_contract_option = ORIGINAL_STORAGE_721.may_load(
                deps.storage,
                (msg.data.source_nft_contract_address.clone(), self_chain),
            )?;
        }
    }

    match storage_contract_option {
        Some(v) => {
            storage_contract = v;
            has_storage = true;
        }
        None => {
            storage_contract = Addr::unchecked("none");
            has_storage = false;
        }
    }
    let res = if has_duplicate && has_storage {
        let is_storage_is_nft_owner_option = NFT_COLLECTION_OWNER.may_load(
            deps.storage,
            (
                duplicate_collection_address
                    .contract_address
                    .clone()
                    .into_string(),
                msg.data.token_id.to_string(),
            ),
        )?;

        match is_storage_is_nft_owner_option {
            Some(_v) => {
                let create_unlock_msg = NftStoreExecuteMsg::UnLockToken {
                    token_id: msg.data.token_id.clone(),
                    to: msg.data.destination_user_address.clone(),
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: storage_contract.clone().into_string(),

                    msg: to_json_binary(&create_unlock_msg)?,
                    funds: vec![],
                });

                let log: Vec<Attribute> = vec![
                    UnLock721EventInfo::new(
                        msg.data.destination_user_address,
                        msg.data.token_id.clone(),
                        storage_contract.to_string(),
                    )
                    .try_into()?,
                    ClaimedEventInfo::new(
                        msg.data.lock_tx_chain,
                        msg.data.source_chain,
                        msg.data.transaction_hash,
                        duplicate_collection_address.contract_address.clone(),
                        msg.data.token_id.clone(),
                    )
                    .try_into()?,
                ];

                let _ = NFT_COLLECTION_OWNER.remove(
                    deps.storage,
                    (
                        duplicate_collection_address.contract_address.into_string(),
                        msg.data.token_id,
                    ),
                );

                Ok(Response::new().add_message(message).add_attributes(log))
            }
            None => {
                let create_collection_msg = NftExecuteMsg::Mint {
                    token_id: msg.data.token_id.to_string(),
                    owner: msg.data.destination_user_address.into_string(),

                    token_uri: Some(msg.data.metadata),
                    extension: RoyaltyData {
                        royalty_percentage: msg.data.royalty as u64,
                        royalty_payment_address: msg.data.royalty_receiver.into_string(),
                    },
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: duplicate_collection_address
                        .contract_address
                        .clone()
                        .into_string(),
                    msg: to_json_binary(&create_collection_msg)?,
                    funds: vec![],
                });
                let log: Vec<Attribute> = vec![ClaimedEventInfo::new(
                    msg.data.lock_tx_chain,
                    msg.data.source_chain,
                    msg.data.transaction_hash,
                    duplicate_collection_address.contract_address,
                    msg.data.token_id,
                )
                .try_into()?];

                Ok(Response::new().add_message(message).add_attributes(log))
            }
        }
    } else if has_duplicate && !has_storage {
        let create_collection_msg = NftExecuteMsg::Mint {
            token_id: msg.data.token_id.to_string(),
            owner: msg.data.destination_user_address.into_string(),
            token_uri: Some(msg.data.metadata),
            extension: RoyaltyData {
                royalty_percentage: msg.data.royalty as u64,
                royalty_payment_address: msg.data.royalty_receiver.into_string(),
            },
        };

        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: duplicate_collection_address
                .contract_address
                .clone()
                .into_string(),
            msg: to_json_binary(&create_collection_msg)?,
            funds: vec![],
        });
        let log: Vec<Attribute> = vec![ClaimedEventInfo::new(
            msg.data.lock_tx_chain,
            msg.data.source_chain,
            msg.data.transaction_hash,
            duplicate_collection_address.contract_address,
            msg.data.token_id,
        )
        .try_into()?];

        Ok(Response::new().add_message(message).add_attributes(log))
    } else if !has_duplicate && !has_storage {
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
        )
    }
    // ===============================/ NOT hasDuplicate && hasStorage /=======================
    else if !has_duplicate && has_storage {
        let is_storage_is_nft_owner_option = NFT_COLLECTION_OWNER.may_load(
            deps.storage,
            (
                msg.data.source_nft_contract_address.clone(),
                msg.data.token_id.to_string(),
            ),
        )?;

        match is_storage_is_nft_owner_option {
            Some(_v) => {
                let create_unlock_msg = NftStoreExecuteMsg::UnLockToken {
                    token_id: msg.data.token_id.clone(),
                    to: msg.data.destination_user_address.clone(),
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: storage_contract.clone().into_string(),

                    msg: to_json_binary(&create_unlock_msg)?,
                    funds: vec![],
                });

                let log: Vec<Attribute> = vec![
                    UnLock721EventInfo::new(
                        msg.data.destination_user_address,
                        msg.data.token_id.clone(),
                        storage_contract.to_string(),
                    )
                    .try_into()?,
                    ClaimedEventInfo::new(
                        msg.data.lock_tx_chain,
                        msg.data.source_chain,
                        msg.data.transaction_hash,
                        Addr::unchecked(msg.data.source_nft_contract_address.clone()),
                        msg.data.token_id.clone(),
                    )
                    .try_into()?,
                ];

                let _ = NFT_COLLECTION_OWNER.remove(
                    deps.storage,
                    (
                        msg.data.source_nft_contract_address.clone(),
                        msg.data.token_id,
                    ),
                );

                Ok(Response::new().add_message(message).add_attributes(log))
            }
            None => {
                let create_collection_msg = NftExecuteMsg::Mint {
                    token_id: msg.data.token_id.to_string(),
                    owner: msg.data.destination_user_address.into_string(),
                    token_uri: Some(msg.data.metadata),
                    extension: RoyaltyData {
                        royalty_percentage: msg.data.royalty as u64,
                        royalty_payment_address: msg.data.royalty_receiver.into_string(),
                    },
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: msg.data.source_nft_contract_address.clone(),

                    msg: to_json_binary(&create_collection_msg)?,
                    funds: vec![],
                });

                let log: Vec<Attribute> = vec![ClaimedEventInfo::new(
                    msg.data.lock_tx_chain,
                    msg.data.source_chain,
                    msg.data.transaction_hash,
                    Addr::unchecked(msg.data.source_nft_contract_address),
                    msg.data.token_id,
                )
                .try_into()?];

                Ok(Response::new().add_message(message).add_attributes(log))
            }
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
        BridgeQueryMsg::GetValidatorsCount {} => to_json_binary(&validators_count(deps)?),
        BridgeQueryMsg::GetValidator { address } => to_json_binary(&validators(deps, address)?),
        BridgeQueryMsg::GetCollectionDeployer {} => to_json_binary(&collection_deployer(deps)?),
        BridgeQueryMsg::GetStorageDeployer {} => to_json_binary(&storage_deployer(deps)?),
        BridgeQueryMsg::GetOriginalStorage721 {
            contract_address,
            chain,
        } => to_json_binary(&original_storage721(deps, contract_address, chain)?),
        BridgeQueryMsg::GetDuplicateStorage721 {
            contract_address,
            chain,
        } => to_json_binary(&duplicate_storage721(deps, contract_address, chain)?),
        BridgeQueryMsg::GetOriginalToDuplicate {
            contract_address,
            chain,
        } => to_json_binary(&original_to_duplicate(deps, contract_address, chain)?),
        BridgeQueryMsg::GetDuplicateToOriginal {
            contract_address,
            chain,
        } => to_json_binary(&duplicate_to_original(deps, contract_address, chain)?),
    }
}

fn validators_count(deps: Deps) -> StdResult<GetValidatorCountResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(GetValidatorCountResponse {
        count: state.validators_count,
    })
}

fn validators(deps: Deps, address: Binary) -> StdResult<GetValidatorResponse> {
    let validator_option = VALIDATORS_STORAGE.may_load(deps.storage, address.0)?;
    Ok(GetValidatorResponse {
        data: validator_option,
    })
}

fn collection_deployer(deps: Deps) -> StdResult<GetCollectionDeployerResponse> {
    let collection_deployer = CONFIG.load(deps.storage)?.collection_deployer;
    Ok(GetCollectionDeployerResponse {
        data: collection_deployer,
    })
}

fn storage_deployer(deps: Deps) -> StdResult<GetStorageDeployerResponse> {
    let storage_deployer = CONFIG.load(deps.storage)?.storage_deployer;
    Ok(GetStorageDeployerResponse {
        data: storage_deployer,
    })
}

fn original_storage721(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<GetStorageResponse> {
    let storage_option = ORIGINAL_STORAGE_721.may_load(deps.storage, (contract_address, chain))?;
    Ok(GetStorageResponse {
        data: storage_option,
    })
}

fn duplicate_storage721(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<GetStorageResponse> {
    let storage_option = DUPLICATE_STORAGE_721.may_load(deps.storage, (contract_address, chain))?;
    Ok(GetStorageResponse {
        data: storage_option,
    })
}

fn original_to_duplicate(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<GetOriginalToDuplicateResponse> {
    let storage_option =
        ORIGINAL_TO_DUPLICATE_STORAGE.may_load(deps.storage, (contract_address, chain))?;
    Ok(GetOriginalToDuplicateResponse {
        data: storage_option,
    })
}

fn duplicate_to_original(
    deps: Deps,
    contract_address: Addr,
    chain: String,
) -> StdResult<GetDuplicateToOriginalResponse> {
    let storage_option =
        DUPLICATE_TO_ORIGINAL_STORAGE.may_load(deps.storage, (contract_address, chain))?;
    Ok(GetDuplicateToOriginalResponse {
        data: storage_option,
    })
}

// Replies
#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        STORAGE_DEPLOYER_REPLY_ID => handle_storage_deployer_reply(_deps, msg),
        STORAGE_DEPLOYER_721_REPLY_ID => handle_storage_reply_721(_deps, msg),
        COLLECTION_DEPLOYER_REPLY_ID => handle_collection_deployer_reply(_deps, msg),
        COLLECTION_DEPLOYER_721_REPLY_ID => handle_collection_reply_721(_deps, msg),
        id => Err(ContractError::UnexpectedReplyId { id }),
    }
}

fn handle_storage_deployer_reply(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let parsed = parse_reply_instantiate_data(msg).map_err(|e| ContractError::CustomError {
        val: format!("Failed to parse instantiate data: {}", e),
    })?;
    match parsed.data {
        Some(bin) => {
            let reply_info: ReplyStorageDeployerInfo = from_json(&bin)?;
            register_storage_deployer_impl(_deps, reply_info)
        }
        None => Err(ContractError::CustomError {
            val: "Init didn't response with storage deployer".to_string(),
        }),
    }
}

fn handle_storage_reply_721(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info = parse_execute_response_data(&bin.0).map_err(|e| {
                    ContractError::CustomError {
                        val: format!("Failed to parse reply data: {}", e),
                    }
                })?;
                match reply_info.data {
                    Some(a) => {
                        let reply_info: ReplyStorageInfo = from_json(&a)?;
                        register_storage_721_impl(_deps, reply_info)
                    }
                    None => Err(ContractError::CustomError {
                        val: "Init didn't response with storage address 721".to_string(),
                    }),
                }
            }
            None => Err(ContractError::CustomError {
                val: "Init didn't response with storage address 721".to_string(),
            }),
        },
        SubMsgResult::Err(e) => Err(ContractError::CustomError { val: e }),
    }
}

fn handle_collection_deployer_reply(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let parsed = parse_reply_instantiate_data(msg).map_err(|e| ContractError::CustomError {
        val: format!("Failed to parse instantiate data: {}", e),
    })?;
    match parsed.data {
        Some(bin) => {
            let reply_info: ReplyCollectionDeployerInfo = from_json(&bin)?;
            register_collection_deployer_impl(_deps, reply_info)
        }
        None => Err(ContractError::CustomError {
            val: "Init didn't response with collection deployer".to_string(),
        }),
    }
}

fn handle_collection_reply_721(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let parsed = cw0::parse_reply_execute_data(msg).map_err(|e| ContractError::CustomError {
        val: format!("Failed to parse execute data: {}", e),
    })?;
    match parsed.data {
        Some(bin) => {
            let reply_info: ReplyCollectionInfo = from_json(&bin)?;
            register_collection_721_impl(_deps, reply_info)
        }
        None => Err(ContractError::CustomError {
            val: "Init didn't response with collection address 721".to_string(),
        }),
    }
}

fn register_storage_deployer_impl(
    deps: DepsMut,
    reply_info: ReplyStorageDeployerInfo,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut state| -> Result<_, StdError> {
        state.storage_deployer = reply_info.address.clone();
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("storage_deployer", &reply_info.address))
}

fn register_storage_721_impl(
    deps: DepsMut,
    reply_info: ReplyStorageInfo,
) -> Result<Response, ContractError> {
    let self_chain = CONFIG.load(deps.storage)?.self_chain;
    if reply_info.is_original {
        let _ = ORIGINAL_STORAGE_721.save(
            deps.storage,
            (reply_info.label.clone(), self_chain),
            &(reply_info.address.clone()),
        );
    } else {
        let _ = DUPLICATE_STORAGE_721.save(
            deps.storage,
            (reply_info.label.clone(), self_chain),
            &reply_info.address.clone(),
        );
    }
    let res = transfer_to_storage_721(
        deps.storage,
        reply_info.address.clone(),
        deps.api.addr_validate(&reply_info.label.clone())?,
        reply_info.token_id,
    )?;
    Ok(res.add_attribute("storage_address_721", &reply_info.address))
}

fn register_collection_deployer_impl(
    deps: DepsMut,
    reply_info: ReplyCollectionDeployerInfo,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut state| -> Result<_, StdError> {
        state.collection_deployer = reply_info.address.clone();
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("collection_deployer", &reply_info.address))
}

fn register_collection_721_impl(
    deps: DepsMut,
    reply_info: ReplyCollectionInfo,
) -> Result<Response, ContractError> {
    let self_chain = CONFIG.load(deps.storage)?.self_chain;

    let _ = ORIGINAL_TO_DUPLICATE_STORAGE.save(
        deps.storage,
        (
            reply_info.source_nft_contract_address.clone(),
            reply_info.source_chain.clone(),
        ),
        &OriginalToDuplicateContractInfo {
            chain: self_chain.clone(),
            contract_address: reply_info.address.clone(),
        },
    );

    let _ = DUPLICATE_TO_ORIGINAL_STORAGE.save(
        deps.storage,
        (reply_info.address.clone(), self_chain),
        &DuplicateToOriginalContractInfo {
            chain: reply_info.source_chain.clone(),
            contract_address: reply_info.source_nft_contract_address,
        },
    );

    let create_collection_msg = NftExecuteMsg::Mint {
        token_id: reply_info.token_id.to_string(),
        owner: reply_info.destination_user_address.into_string(),
        extension: RoyaltyData {
            royalty_payment_address: reply_info.royalty_receiver.to_string(),
            royalty_percentage: reply_info.royalty as u64,
        },
        token_uri: Some(reply_info.metadata),
    };

    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: reply_info.address.clone().into_string(),
        msg: to_json_binary(&create_collection_msg)?,
        funds: vec![],
    });
    let emit: Vec<Attribute> = vec![ClaimedEventInfo::new(
        reply_info.lock_tx_chain,
        reply_info.source_chain,
        reply_info.transaction_hash,
        reply_info.address.clone(),
        reply_info.token_id,
    )
    .try_into()?];

    Ok(Response::new()
        .add_message(message)
        .add_attributes(emit)
        .add_attribute("collection_address_721", &reply_info.address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{from_json, testing::*};
    use cosmwasm_std::{Coin, Uint128};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let info = mock_info(
            "creator",
            &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::new(1000),
            }],
        );

        let validator_pub_key =
            to_json_binary(&"secret1w5fw0m5cad30lsu8x65m57ad5s80f0fmg3jfal".to_string()).unwrap();
        let init_msg = BridgeInstantiateMsg {
            validators: vec![(validator_pub_key.clone(), info.sender.clone())],
            chain_type: "SECRET".to_string(),
            storage_label: "storage11".to_string(),
            collection_label: "collection11".to_string(),
            collection721_code_id: 1,
            storage721_code_id: 2,
            collection_deployer_code_id: 5,
            storage_deployer_code_id: 6,
        };

        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

        assert_eq!(2, res.messages.len());

        let validators_count_binary = query(
            deps.as_ref(),
            mock_env(),
            BridgeQueryMsg::GetValidatorsCount {},
        )
        .unwrap();

        let validator_info_binary = query(
            deps.as_ref(),
            mock_env(),
            BridgeQueryMsg::GetValidator {
                address: validator_pub_key,
            },
        )
        .unwrap();

        let collection_deployer_binary = query(
            deps.as_ref(),
            mock_env(),
            BridgeQueryMsg::GetCollectionDeployer {},
        )
        .unwrap();

        let storage_deployer_binary = query(
            deps.as_ref(),
            mock_env(),
            BridgeQueryMsg::GetStorageDeployer {},
        )
        .unwrap();

        let validators_count_answer =
            from_json::<GetValidatorCountResponse>(&validators_count_binary).unwrap();
        let validator_answer = from_json::<GetValidatorResponse>(&validator_info_binary).unwrap();
        let collection_deployer_answer =
            from_json::<GetCollectionDeployerResponse>(&collection_deployer_binary).unwrap();
        let storage_deployer_answer =
            from_json::<GetStorageDeployerResponse>(&storage_deployer_binary).unwrap();
        assert_eq!(
            1, validators_count_answer.count,
            "Invalid validator count after init"
        );
        assert_eq!(
            true,
            validator_answer.data.unwrap().added,
            "Invalid validator after init"
        );

        let valid_addr = deps
            .api
            .addr_validate(&collection_deployer_answer.data.into_string());
        assert!(valid_addr.is_ok(), "Invalid collection deployer after init");

        let valid_addr = deps
            .api
            .addr_validate(&storage_deployer_answer.data.into_string());
        assert!(valid_addr.is_ok(), "Invalid storage deployer after init");
    }
}
