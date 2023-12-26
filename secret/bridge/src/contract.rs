use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Api, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, StdResult, Storage, SubMsg,
    SubMsgResult, Uint128, WasmMsg,
};
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::snip721::Metadata;
use secret_toolkit::storage::{Keymap, WithoutIter};
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};

use crate::collection_deployer_msg::{
    CollectionDeployerExecuteMsg, CurateTokenId, InstantiateCollectionDeployer, TknConfig,
    TokenIdBalance, TokenInfoMsg,
};
use crate::error::ContractError;
use crate::events::{
    AddNewValidatorEventInfo, ClaimedEventInfo, LockedEventInfo, RewardValidatorEventInfo,
    UnLock1155EventInfo, UnLock721EventInfo,
};
use crate::msg::{ExecuteMsg, QueryAnswer, QueryMsg};
use crate::snip1155_msg::{
    Collection1155OwnerOfResponse, Collection1155QueryMsg, Snip1155ExecuteMsg, TokenAmount,
};
use crate::snip721_msg::Snip721ExecuteMsg;
use crate::state::{
    config, config_read, CODEHASHES, COLLECTION_DEPLOYER_1155_REPLY_ID,
    COLLECTION_DEPLOYER_721_REPLY_ID, COLLECTION_DEPLOYER_REPLY_ID, COLLETION_DEPLOYER_CODE,
    DUPLICATE_STORAGE_1155, DUPLICATE_STORAGE_721, DUPLICATE_TO_ORIGINAL_STORAGE,
    NFT_COLLECTION_OWNER, ORIGINAL_STORAGE_1155, ORIGINAL_STORAGE_721,
    ORIGINAL_TO_DUPLICATE_STORAGE, STORAGE_DEPLOYER_1155_REPLY_ID, STORAGE_DEPLOYER_721_REPLY_ID,
    STORAGE_DEPLOYER_CODE, STORAGE_DEPLOYER_REPLY_ID, UNIQUE_IDENTIFIER_STORAGE,
    VALIDATORS_STORAGE,
};
use crate::storage1155_msg::Storage1155ExecuteMsg;
use crate::storage721_msg::Storage721ExecuteMsg;
use crate::storage_deployer_msg::{InstantiateStorageDeployer, StorageDeployerExecuteMsg};
use crate::structs::{
    AddValidatorMsg, ClaimData, ClaimMsg, ClaimValidatorRewardsMsg, CodeInfo,
    DuplicateToOriginalContractInfo, InstantiateMsg, Lock1155Msg, Lock721Msg,
    OriginalToDuplicateContractInfo, ReplyCollectionDeployerInfo, ReplyCollectionInfo,
    ReplyStorageDeployerInfo, ReplyStorageInfo, Royalty, RoyaltyInfo, SignerAndSignature, State,
    Validator, VerifyMsg,
};
use sha2::{Digest, Sha256};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api
        .debug(format!("Contract was initialized by {}", info.sender).as_str());

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
        storage_deployer: _env.contract.address,
        validators_count,
        self_chain: msg.chain_type,
        type_erc_721: "singular".to_owned(),
        type_erc_1155: "multiple".to_owned(),
    };

    config(deps.storage).save(&state)?;

    // config(deps.storage).update(|mut state| -> Result<_, StdError> {
    //     for ele in msg.validators {
    //         let mut validatorEntry: BTreeMap<Addr, Validator> = BTreeMap::new();
    //         validatorEntry.insert(
    //             ele,
    //             Validator {
    //                 added: true,
    //                 pending_reward: 0,
    //             },
    //         );
    //         state.validators_count += 1;
    //         state.validators.append(&mut validatorEntry);
    //     }
    //     Ok(state)
    // })?;

    STORAGE_DEPLOYER_CODE.save(deps.storage, &msg.storage_deployer_code_info)?;
    COLLETION_DEPLOYER_CODE.save(deps.storage, &msg.collection_deployer_code_info)?;

    let init_storage_deployer_msg = InstantiateStorageDeployer {
        storage721_code_info: msg.storage721_code_info,
        storage1155_code_info: msg.storage1155_code_info,
    };

    let init_storage_deployer_sub_msg = SubMsg::reply_always(
        init_storage_deployer_msg.to_cosmos_msg(
            msg.storage_label,
            msg.storage_deployer_code_info.code_id,
            msg.storage_deployer_code_info.code_hash,
            None,
        )?,
        STORAGE_DEPLOYER_REPLY_ID,
    );

    let init_collection_deployer_msg = InstantiateCollectionDeployer {
        collection721_code_info: msg.collection721_code_info,
        collection1155_code_info: msg.collection1155_code_info,
    };

    let init_collection_deployer_submsg = SubMsg::reply_always(
        init_collection_deployer_msg.to_cosmos_msg(
            msg.collection_label,
            msg.collection_deployer_code_info.code_id,
            msg.collection_deployer_code_info.code_hash,
            None,
        )?,
        COLLECTION_DEPLOYER_REPLY_ID,
    );
    Ok(Response::new().add_submessages(vec![
        init_storage_deployer_sub_msg,
        init_collection_deployer_submsg,
    ]))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    // let response = match msg {
    //     ExecuteMsg::CreateOffspring {
    //         label,
    //         owner,
    //         count,
    //         description,
    //     } => try_create_offspring(deps, env, label, owner, count, description),
    //     ExecuteMsg::DeactivateOffspring { owner } => try_deactivate_offspring(deps, info, owner),
    //     ExecuteMsg::CreateViewingKey { entropy } => try_create_key(deps, env, info, entropy),
    //     ExecuteMsg::SetViewingKey { key, .. } => try_set_key(deps, info, &key),
    //     ExecuteMsg::NewOffspringContract {
    //         offspring_code_info,
    //     } => try_new_contract(deps, info, offspring_code_info),
    //     ExecuteMsg::SetStatus { stop } => try_set_status(deps, info, stop),
    //     ExecuteMsg::RevokePermit { permit_name, .. } => revoke_permit(deps, info, permit_name),
    // };
    // pad_handle_result(response, BLOCK_SIZE)

    match msg {
        ExecuteMsg::AddValidator { data } => add_validator(deps, data),
        ExecuteMsg::ClaimValidatorRewards { data } => claim_validator_rewards(deps, data),
        ExecuteMsg::Lock721 { data } => lock721(deps, env, data),
        ExecuteMsg::Lock1155 { data } => lock1155(deps, env, info, data),
        ExecuteMsg::Claim721 { data } => claim721(deps, env, info, data),
        ExecuteMsg::Claim1155 { data } => claim1155(deps, env, info, data),
        ExecuteMsg::VerifySig { data } => verify_sig(deps, data),
    }
}

fn verify_sig(deps: DepsMut, msg: VerifyMsg) -> StdResult<Response> {
    let serialized = serde_json::to_vec(&msg.claim_data_as_binary).unwrap();
    let mut hasher = Sha256::new();
    let mut hash = [0u8; 32];
    hasher.update(serialized);
    hash = hasher.finalize().into();

    deps.api
        .debug(format!("AAAAAAAAAAAAAAAA {:?}", hash).as_str());

    let serialized = msg.claim_data_as_binary.concat_all_fields();
    let mut hasher = Sha256::new();
    let mut hash = [0u8; 32];
    hasher.update(serialized);
    hash = hasher.finalize().into();

    deps.api
        .debug(format!("BBBBBBBBBBBBBBBB {:?}", hash).as_str());

    let serialized = msg.msg_as_bindary;
    let mut hasher = Sha256::new();
    let mut hash = [0u8; 32];
    hasher.update(&*serialized);
    hash = hasher.finalize().into();

    deps.api
        .debug(format!("DDDDDDDDDDDDDDDD {:?}", hash).as_str());

    deps.api
        .debug(format!("EEEEEEEEEEEEEEEE {:?}", msg.claim_data).as_str());

    let res = deps
        .api
        .secp256k1_verify(&msg.claim_data, &msg.signature, &msg.user)?;
    if !res {
        Err(StdError::generic_err(
            "Signature verification unsuccessful".to_string(),
        ))
    } else {
        deps.api.debug(format!("fffffffffffffff {}", res).as_str());

        Ok(Response::default())
    }
}

fn matches_current_chain(storage: &dyn Storage, destination_chain: String) -> StdResult<Response> {
    if destination_chain == config_read(storage).load()?.self_chain {
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

fn add_validator(deps: DepsMut, add_validator_msg: AddValidatorMsg) -> StdResult<Response> {
    if add_validator_msg.signatures.is_empty() {
        return Err(StdError::generic_err("Must have signatures!"));
    }

    let state = config_read(deps.storage).load()?;
    if VALIDATORS_STORAGE.contains(deps.storage, &add_validator_msg.validator.0) {
        return Err(StdError::generic_err("Validator already added"));
    }

    let percentage = validate_signatures(
        deps.api,
        &add_validator_msg.validator.0,
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
    let mut hash = [0u8; 32];
    hasher.update(serialized);
    hash = hasher.finalize().into();

    for ele in sigs {
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
        // let sig_array = <[u8; 64]>::try_from(signature)
        //     .map_err(|_| StdError::generic_err("Invalid signature length"))?;
        // let signer_array = <[u8; 32]>::try_from(signer_address)
        //     .map_err(|_| StdError::generic_err("Invalid signer address length"))?;
        // let sig = Signature::new(sig_array);
        // let key = PublicKey::new(signer_array);

        // key.verify(validator.as_ref(), &sig)
        //     .map(|_| true)
        //     .map_err(|_| StdError::generic_err("Signature verification failed"))

        let result = api.secp256k1_verify(hash, &signature, &signer_address)?;
        api.debug(format!("SIG RESULT {}", result).as_str());
        if result {
            api.debug(format!("TRUE SIG {}", true).as_str());
            Ok(true)
        } else {
            api.debug(format!("FALSE SIG {}", true).as_str());
            Ok(false)
        }
    } else {
        api.debug(format!("TFT SIG {}", true).as_str());
        Ok(false)
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
    if data.signatures.is_empty() {
        return Err(StdError::generic_err("Must have signatures!"));
    }

    let state = config_read(deps.storage).load()?;

    if !VALIDATORS_STORAGE
        .get(deps.storage, &data.validator)
        .is_some()
    {
        return Err(StdError::generic_err("Validator does not exist!"));
    }

    let percentage = validate_signatures(deps.api, &data.validator, &data.signatures)?;
    if percentage < required_threshold(state.validators_count as u128) {
        return Err(StdError::generic_err("Threshold not reached!"));
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
) -> StdResult<Response> {
    let storage_address_option = storage_mapping_721.get(
        deps.storage,
        &(
            source_nft_contract_address.clone().into_string(),
            self_chain.clone(),
        ),
    );

    // let transfer_msg = Snip721ExecuteMsg::TransferNft {
    //     recipient: owner.clone().into_string(),
    //     token_id: token_id.to_string(),
    //     memo: Option::None,
    //     padding: Option::None,
    // }
    // .to_cosmos_msg(
    //     collection_code_info.code_hash.clone(),
    //     source_nft_contract_address.clone().into_string(),
    //     None,
    // )?;

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
            let create_storage_msg = StorageDeployerExecuteMsg::CreateStorage721 {
                label: source_nft_contract_address.clone().into_string(),
                collection_address: source_nft_contract_address.clone(),
                collection_code_info,
                owner: owner.into_string(),
                is_original,
                token_id,
            };

            let code_info = STORAGE_DEPLOYER_CODE.load(deps.storage)?;

            deps.api
                .debug(format!("LOCK721 {}", code_info.code_id).as_str());

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
    let transfer_msg = Snip721ExecuteMsg::TransferNft {
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
) -> StdResult<Response> {
    let storage_address_option = storage_mapping_1155.get(
        deps.storage,
        &(
            source_nft_contract_address.clone().into_string(),
            self_chain,
        ),
    );

    // let transfer_msg = Snip1155ExecuteMsg::Transfer {
    //     token_id: token_id.to_string(),
    //     from,
    //     recipient: owner.clone(),
    //     amount: token_amount.into(),
    //     memo: Option::None,
    //     padding: Option::None,
    // }
    // .to_cosmos_msg(
    //     collection_code_info.code_hash.clone(),
    //     source_nft_contract_address.clone().into_string(),
    //     None,
    // )?;

    let _ = CODEHASHES.insert(
        deps.storage,
        &source_nft_contract_address.clone(),
        &collection_code_info.code_hash.clone(),
    );

    match storage_address_option {
        Some(v) => transfer_to_storage_1155(
            deps.storage,
            owner,
            v.0,
            source_nft_contract_address.clone(),
            token_id,
            token_amount,
            collection_code_info.code_hash,
        ),
        None => {
            let create_storage_msg = StorageDeployerExecuteMsg::CreateStorage1155 {
                label: source_nft_contract_address.clone().into_string(),
                collection_address: source_nft_contract_address.clone(),
                collection_code_info,
                owner: owner.into_string(),
                is_original,
                token_id,
            };

            let code_info = STORAGE_DEPLOYER_CODE.load(deps.storage)?;

            deps.api
                .debug(format!("LOCK1155 {}", code_info.code_id).as_str());

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
    from: Addr,
    storage_address: Addr,
    source_nft_contract_address: Addr,
    token_id: String,
    token_amount: u128,
    code_hash: String,
) -> StdResult<Response> {
    let transfer_msg = Snip1155ExecuteMsg::Transfer {
        token_id: token_id.to_string(),
        from,
        recipient: storage_address.clone(),
        amount: token_amount.into(),
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
        &(storage_address, token_amount),
    );

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
            )?;
            Ok(res.add_attributes(log))
        }
    }
}

fn create_claim_data_hash(data: ClaimData) -> [u8; 32] {
    let serialized = data.concat_all_fields();
    let mut hasher = Sha256::new();
    let mut output = [0u8; 32];
    hasher.update(serialized);
    output = hasher.finalize().into();

    output
}

fn verify_signatures(
    api: &dyn Api,
    signature: &[u8],
    signer_address: &[u8],
    hash: &[u8; 32],
) -> StdResult<bool> {
    if signature.len() == 64 && signer_address.len() == 33 {
        let result = api.secp256k1_verify(hash, &signature, &signer_address)?;
        if result {
            api.debug(format!("TRUE SIG {}", true).as_str());
            Ok(true)
        } else {
            api.debug(format!("FALSE SIG {}", true).as_str());
            Ok(false)
        }
    } else {
        api.debug(format!("TFT SIG {}", true).as_str());
        Ok(false)
    }
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

    api.debug(
        format!(
            "PERCENTAGE {}, {}",
            percentage,
            required_threshold(validators_count)
        )
        .as_str(),
    );

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

    let fee_per_validator = balance / validators_to_reward.len() as u128;

    for val in validators_to_reward {
        let validator_option = VALIDATORS_STORAGE.get(storage, &val);
        match validator_option {
            Some(mut v) => {
                v.pending_reward = v.pending_reward + fee_per_validator;

                let _ = VALIDATORS_STORAGE.insert(storage, &val, &v);
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
    };

    let code_info = COLLETION_DEPLOYER_CODE.load(deps.storage)?;

    deps.api
        .debug(format!("create 721 collection {}", code_info.code_id).as_str());

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
) -> StdResult<Response> {
    let create_collection_msg = CollectionDeployerExecuteMsg::CreateCollection1155 {
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
                // {
                //     minters: vec![owner.clone()],
                //     public_total_supply: true,
                //     owner_is_public: false,
                //     enable_burn: false,
                //     owner_may_update_metadata: true,
                //     minter_may_update_metadata: true,
                // },
                public_metadata: Some(Metadata {
                    token_uri: Some(metadata.clone()),
                    extension: Option::None,
                }),
                private_metadata: Option::None,
            },
            balances: vec![TokenIdBalance {
                address: owner,
                amount: Uint128::from(1u128),
            }],
        }],
        entropy: name.clone() + &symbol,
        label: name.clone() + &symbol,
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
    };

    let code_info = COLLETION_DEPLOYER_CODE.load(deps.storage)?;

    deps.api
        .debug(format!("create 1155 collection {}", code_info.code_id).as_str());

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
    deps.api.debug(format!("CLAIM721 started").as_str());

    let balance = deps
        .querier
        .query_balance(env.contract.address.clone(), "uscrt".to_string())?
        .amount;

    deps.api
        .debug(format!("CONTRACT BALANCE  {}", balance).as_str());

    let self_chain = config_read(deps.storage).load()?.self_chain;

    deps.api
        .debug(format!("SELF CHAIN  {}", self_chain).as_str());

    let _ = has_correct_fee(msg.data.fee.clone(), info);

    let type_erc_721 = config_read(deps.storage).load()?.type_erc_721;

    deps.api
        .debug(format!("TYPEERC721  {}", type_erc_721).as_str());

    let validators_count = config_read(deps.storage).load()?.validators_count;

    deps.api
        .debug(format!("VALIDATOR COUNT  {}", validators_count).as_str());

    let _ = matches_current_chain(deps.storage, msg.data.destination_chain.clone());

    if msg.data.nft_type != type_erc_721 {
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

    deps.api
        .debug(format!("DETAILS {} {}", has_duplicate, has_storage).as_str());

    match storage_contract_option {
        Some(v) => {
            storage_contract = v;
            has_storage = true;
        }
        None => {
            storage_contract = (Addr::unchecked("none"), "none".to_string());
            has_storage = false;
            deps.api
                .debug(format!("DETAILS {} {}", has_duplicate, has_storage).as_str());
        }
    }
    deps.api
        .debug(format!("DETAILS {} {}", has_duplicate, has_storage).as_str());
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
                let create_unlock_msg = Storage721ExecuteMsg::UnLockToken {
                    token_id: msg.data.token_id.clone(),
                    to: msg.data.destination_user_address.clone(),
                };

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: storage_contract.0.clone().into_string(),
                    code_hash: storage_contract.1,
                    msg: to_binary(&create_unlock_msg)?,
                    funds: vec![],
                });

                let log: Vec<Attribute> = vec![UnLock721EventInfo::new(
                    msg.data.destination_user_address,
                    msg.data.token_id.clone(),
                    storage_contract.0.to_string(),
                )
                .try_into()?];

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
                let create_collection_msg = Snip721ExecuteMsg::MintNft {
                    token_id: Some(msg.data.token_id.to_string()),
                    owner: Some(msg.data.destination_user_address.into_string()),
                    public_metadata: Some(Metadata {
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
                    contract_addr: duplicate_collection_address.contract_address.into_string(),
                    code_hash: duplicate_collection_address.code_hash,
                    msg: to_binary(&create_collection_msg)?,
                    funds: vec![],
                });

                Ok(Response::new().add_message(message))
            }
        }
    }
    // ===============================/ hasDuplicate && NOT hasStorage /=======================
    else if has_duplicate && !has_storage {
        let create_collection_msg = Snip721ExecuteMsg::MintNft {
            token_id: Some(msg.data.token_id.to_string()),
            owner: Some(msg.data.destination_user_address.into_string()),
            public_metadata: Some(Metadata {
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

        deps.api
            .debug(format!("mint 721 nft {}", msg.data.token_id).as_str());

        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: duplicate_collection_address.contract_address.into_string(),
            code_hash: duplicate_collection_address.code_hash,
            msg: to_binary(&create_collection_msg)?,
            funds: vec![],
        });

        Ok(Response::new().add_message(message))
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
            .unwrap();

        let is_storage_is_nft_owner_option = NFT_COLLECTION_OWNER.get(
            deps.storage,
            &(
                msg.data.source_nft_contract_address.clone(),
                msg.data.token_id.to_string(),
            ),
        );

        match is_storage_is_nft_owner_option {
            Some(_v) => {
                let create_unlock_msg = Storage721ExecuteMsg::UnLockToken {
                    token_id: msg.data.token_id.clone(),
                    to: msg.data.destination_user_address.clone(),
                };

                deps.api
                    .debug(format!("unlock 721 nft {}", msg.data.token_id).as_str());

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: storage_contract.0.clone().into_string(),
                    code_hash: storage_contract.1,
                    msg: to_binary(&create_unlock_msg)?,
                    funds: vec![],
                });

                let log: Vec<Attribute> = vec![UnLock721EventInfo::new(
                    msg.data.destination_user_address,
                    msg.data.token_id.clone(),
                    storage_contract.0.to_string(),
                )
                .try_into()?];

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
                let create_collection_msg = Snip721ExecuteMsg::MintNft {
                    token_id: Some(msg.data.token_id.to_string()),
                    owner: Some(msg.data.destination_user_address.into_string()),
                    public_metadata: Some(Metadata {
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

                deps.api
                    .debug(format!("mint 721 nft {}", msg.data.token_id).as_str());

                let message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: msg.data.source_nft_contract_address,
                    code_hash,
                    msg: to_binary(&create_collection_msg)?,
                    funds: vec![],
                });

                Ok(Response::new().add_message(message))
            }
        }
    } else {
        return Err(StdError::generic_err("Invalid bridge state"));
    }?;

    let log: Vec<Attribute> =
        vec![ClaimedEventInfo::new(msg.data.source_chain, msg.data.transaction_hash).try_into()?];

    Ok(res.add_attributes(log))
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
    let mut has_storage: bool = false;
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
        // let get_owner_of = Collection1155QueryMsg::Balance {
        //     owner: storage_contract.0.clone(),
        //     viewer: storage_contract.0.clone(),
        //     key: "key".to_string(),
        //     token_id: msg.data.token_id.to_string(),
        // };
        // let code_hash = duplicate_collection_address.code_hash.clone();
        // let duplicate_collection: Collection1155OwnerOfResponse = get_owner_of.query(
        //     deps.querier,
        //     code_hash,
        //     duplicate_collection_address
        //         .contract_address
        //         .clone()
        //         .into_string(),
        // )?;

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
                    let create_unlock_msg = Storage1155ExecuteMsg::UnLockToken {
                        token_id: msg.data.token_id.clone(),
                        to: msg.data.destination_user_address.clone(),
                        amount: msg.data.token_amount.clone(),
                    };

                    deps.api
                        .debug(format!("unlock 1155 nft {}", msg.data.token_id).as_str());

                    let message = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                        contract_addr: storage_contract.0.clone().into_string(),
                        code_hash: storage_contract.1,
                        msg: to_binary(&create_unlock_msg)?,
                        funds: vec![],
                    });

                    let log: Vec<Attribute> = vec![UnLock1155EventInfo::new(
                        msg.data.destination_user_address,
                        msg.data.token_id.clone(),
                        storage_contract.0.to_string(),
                        msg.data.token_amount,
                    )
                    .try_into()?];

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
                        Uint128::from(msg.data.token_amount.clone()) - Uint128::from(_v.1);

                    let create_unlock_msg = Storage1155ExecuteMsg::UnLockToken {
                        token_id: msg.data.token_id.clone(),
                        to: msg.data.destination_user_address.clone(),
                        amount: _v.1,
                    };

                    deps.api
                        .debug(format!("unlock 1155 nft {}", msg.data.token_id).as_str());

                    let message_unlock = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                        contract_addr: storage_contract.0.clone().into_string(),
                        code_hash: storage_contract.1,
                        msg: to_binary(&create_unlock_msg)?,
                        funds: vec![],
                    });

                    let log: Vec<Attribute> = vec![UnLock1155EventInfo::new(
                        msg.data.destination_user_address.clone(),
                        msg.data.token_id.clone(),
                        storage_contract.0.to_string(),
                        _v.1,
                    )
                    .try_into()?];

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

                    deps.api
                        .debug(format!("mint 1155 nft {}", msg.data.token_id).as_str());

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
            None => todo!(),
        }
    }
    // ===============================/ hasDuplicate && NOT hasStorage /=======================
    else if has_duplicate && !has_storage {
        let create_collection_msg = Snip1155ExecuteMsg::MintTokens {
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

        deps.api
            .debug(format!("mint 1155 nft {}", msg.data.token_id).as_str());

        let message = CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
            contract_addr: duplicate_collection_address.contract_address.into_string(),
            code_hash: duplicate_collection_address.code_hash,
            msg: to_binary(&create_collection_msg)?,
            funds: vec![],
        });

        Ok(Response::new().add_message(message))
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
        )
    }
    // ===============================/ NOT hasDuplicate && hasStorage /=======================
    else if !has_duplicate && has_storage {
        let get_owner_of = Collection1155QueryMsg::Balance {
            owner: storage_contract.0.clone(),
            viewer: storage_contract.0.clone(),
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
        let original_collection: Collection1155OwnerOfResponse = get_owner_of.query(
            deps.querier,
            code_hash.clone(),
            msg.data.source_nft_contract_address.clone(),
        )?;

        if original_collection.amount >= msg.data.token_amount.into() {
            let create_unlock_msg = Storage1155ExecuteMsg::UnLockToken {
                token_id: msg.data.token_id.clone(),
                amount: msg.data.token_amount,
                to: msg.data.destination_user_address.clone(),
            };

            deps.api
                .debug(format!("unlock 1155 nft {}", msg.data.token_id).as_str());

            let message = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: storage_contract.0.clone().into_string(),
                code_hash: storage_contract.1,
                msg: to_binary(&create_unlock_msg)?,
                funds: vec![],
            });

            let log: Vec<Attribute> = vec![UnLock1155EventInfo::new(
                msg.data.destination_user_address.clone(),
                msg.data.token_id.clone(),
                storage_contract.0.to_string(),
                msg.data.token_amount,
            )
            .try_into()?];

            let _ = NFT_COLLECTION_OWNER.remove(
                deps.storage,
                &(
                    msg.data.source_nft_contract_address.clone(),
                    msg.data.token_id.to_string(),
                ),
            );

            Ok(Response::new().add_message(message).add_attributes(log))
        } else {
            let to_mint = Uint128::from(msg.data.token_amount.clone()) - original_collection.amount;

            let create_unlock_msg = Storage1155ExecuteMsg::UnLockToken {
                token_id: msg.data.token_id.clone(),
                amount: original_collection.amount.into(),
                to: msg.data.destination_user_address.clone(),
            };

            deps.api
                .debug(format!("unlock 1155 nft {}", msg.data.token_id).as_str());

            let message_unlock = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: storage_contract.0.clone().into_string(),
                code_hash: storage_contract.1,
                msg: to_binary(&create_unlock_msg)?,
                funds: vec![],
            });

            let log: Vec<Attribute> = vec![UnLock1155EventInfo::new(
                msg.data.destination_user_address.clone(),
                msg.data.token_id.clone(),
                storage_contract.0.to_string(),
                original_collection.amount.into(),
            )
            .try_into()?];

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

            deps.api
                .debug(format!("mint 1155 nft {}", msg.data.token_id).as_str());

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

    let log: Vec<Attribute> =
        vec![ClaimedEventInfo::new(msg.data.source_chain, msg.data.transaction_hash).try_into()?];

    Ok(res.add_attributes(log))
}
// Queries
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetValidatorsCount {} => to_binary(&validators_count(deps)?),
        QueryMsg::GetValidator { address } => to_binary(&validators(deps, address)?),
        QueryMsg::GetCollectionDeployer {} => to_binary(&collection_deployer(deps)?),
        QueryMsg::GetStorageDeployer {} => to_binary(&storage_deployer(deps)?),
        QueryMsg::GetOriginalStorage721 {
            contract_address,
            chain,
        } => to_binary(&original_storage721(deps, contract_address, chain)?),
        QueryMsg::GetDuplicateStorage721 {
            contract_address,
            chain,
        } => to_binary(&duplicate_storage721(deps, contract_address, chain)?),
        QueryMsg::GetOriginalToDuplicate {
            contract_address,
            chain,
        } => to_binary(&original_to_duplicate(deps, contract_address, chain)?),
        QueryMsg::GetDuplicateToOriginal {
            contract_address,
            chain,
        } => to_binary(&duplicate_to_original(deps, contract_address, chain)?),
    }
}

fn validators_count(deps: Deps) -> StdResult<QueryAnswer> {
    let state = config_read(deps.storage).load()?;
    Ok(QueryAnswer::ValidatorCountResponse {
        count: state.validators_count,
    })
}

fn validators(deps: Deps, address: Binary) -> StdResult<QueryAnswer> {
    let validator_option = VALIDATORS_STORAGE.get(deps.storage, &address);
    Ok(QueryAnswer::Validator {
        data: validator_option,
    })
}

fn collection_deployer(deps: Deps) -> StdResult<QueryAnswer> {
    let collection_deployer = config_read(deps.storage).load()?.collection_deployer;
    Ok(QueryAnswer::CollectionDeployer {
        data: collection_deployer,
    })
}

fn storage_deployer(deps: Deps) -> StdResult<QueryAnswer> {
    let storage_deployer = config_read(deps.storage).load()?.storage_deployer;
    Ok(QueryAnswer::StorageDeployer {
        data: storage_deployer,
    })
}

fn original_storage721(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<QueryAnswer> {
    let storage_option = ORIGINAL_STORAGE_721.get(deps.storage, &(contract_address, chain));
    Ok(QueryAnswer::Storage {
        data: storage_option,
    })
}

fn duplicate_storage721(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<QueryAnswer> {
    let storage_option = DUPLICATE_STORAGE_721.get(deps.storage, &(contract_address, chain));
    Ok(QueryAnswer::Storage {
        data: storage_option,
    })
}

fn original_to_duplicate(
    deps: Deps,
    contract_address: String,
    chain: String,
) -> StdResult<QueryAnswer> {
    let storage_option =
        ORIGINAL_TO_DUPLICATE_STORAGE.get(deps.storage, &(contract_address, chain));
    Ok(QueryAnswer::OriginalToDuplicate {
        data: storage_option,
    })
}

fn duplicate_to_original(
    deps: Deps,
    contract_address: Addr,
    chain: String,
) -> StdResult<QueryAnswer> {
    let storage_option =
        DUPLICATE_TO_ORIGINAL_STORAGE.get(deps.storage, &(contract_address, chain));
    Ok(QueryAnswer::DuplicateToOriginal {
        data: storage_option,
    })
}

// Replies
#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    _deps
        .api
        .debug(format!("REPLY FROM STORAGE DEPLOYER  {}", msg.id).as_str());
    match msg.id {
        STORAGE_DEPLOYER_REPLY_ID => handle_storage_deployer_reply(_deps, msg),
        STORAGE_DEPLOYER_721_REPLY_ID => handle_storage_reply_721(_deps, msg),
        STORAGE_DEPLOYER_1155_REPLY_ID => {
            handle_storage_reply_1155(_deps, msg, _env.contract.address)
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
                let reply_info: ReplyStorageInfo = from_binary(&bin)?;
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
    from: Addr,
) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(s) => match s.data {
            Some(bin) => {
                let reply_info: ReplyStorageInfo = from_binary(&bin)?;
                register_storage_1155_impl(_deps, reply_info, from)
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
                let reply_info: ReplyCollectionInfo = from_binary(&bin)?;
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
                let reply_info: ReplyCollectionInfo = from_binary(&bin)?;
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
    reply_info: ReplyStorageInfo,
) -> Result<Response, ContractError> {
    let self_chain = config(deps.storage).load()?.self_chain;
    if reply_info.is_original {
        let _ = ORIGINAL_STORAGE_721.insert(
            deps.storage,
            &(reply_info.label.clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    } else {
        let _ = DUPLICATE_STORAGE_721.insert(
            deps.storage,
            &(reply_info.label.clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    }
    let res = transfer_to_storage_721(
        deps.storage,
        reply_info.address.clone(),
        deps.api.addr_validate(&reply_info.label.clone())?,
        reply_info.token_id,
        reply_info.collection_code_hash,
    )?;
    Ok(res.add_attribute("storage_address_721", &reply_info.address))
}

fn register_storage_1155_impl(
    deps: DepsMut,
    reply_info: ReplyStorageInfo,
    from: Addr,
) -> Result<Response, ContractError> {
    let self_chain = config(deps.storage).load()?.self_chain;
    if reply_info.is_original {
        let _ = ORIGINAL_STORAGE_1155.insert(
            deps.storage,
            &(reply_info.label.clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    } else {
        let _ = DUPLICATE_STORAGE_1155.insert(
            deps.storage,
            &(reply_info.label.clone(), self_chain),
            &(reply_info.address.clone(), reply_info.code_hash.clone()),
        );
    }
    let res = transfer_to_storage_1155(
        deps.storage,
        from,
        reply_info.address.clone(),
        deps.api.addr_validate(&reply_info.label.clone())?,
        reply_info.token_id,
        reply_info.token_amount,
        reply_info.collection_code_hash,
    )?;
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
    reply_info: ReplyCollectionInfo,
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
            chain: reply_info.source_chain,
            contract_address: reply_info.source_nft_contract_address,
            code_hash: "".to_string(),
        },
    );

    let create_collection_msg = Snip721ExecuteMsg::MintNft {
        token_id: Some(reply_info.token_id.to_string()),
        owner: Some(reply_info.destination_user_address.into_string()),
        public_metadata: Some(Metadata {
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

    deps.api
        .debug(format!("mint 721 nft {}", reply_info.token_id).as_str());

    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: reply_info.address.clone().into_string(),
        code_hash: reply_info.code_hash,
        msg: to_binary(&create_collection_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(message)
        .add_attribute("collection_address_721", &reply_info.address))
}

fn register_collection_1155_impl(
    deps: DepsMut,
    reply_info: ReplyCollectionInfo,
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
            chain: reply_info.source_chain,
            contract_address: reply_info.source_nft_contract_address,
            code_hash: "".to_string(),
        },
    );

    let create_collection_msg = Snip1155ExecuteMsg::MintTokens {
        mint_tokens: vec![TokenAmount {
            token_id: reply_info.token_id.to_string(),
            balances: vec![TokenIdBalance {
                address: reply_info.destination_user_address.clone(),
                amount: reply_info.token_amount.into(),
            }],
        }],
        memo: Option::None,
        padding: Option::None,
    };

    deps.api
        .debug(format!("mint 1155 nft {}", reply_info.token_id).as_str());

    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: reply_info.address.clone().into_string(),
        code_hash: reply_info.code_hash,
        msg: to_binary(&create_collection_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(message)
        .add_attribute("collection_address_1155", &reply_info.address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::{from_binary, Coin, Uint128};

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
            to_binary(&"secret1w5fw0m5cad30lsu8x65m57ad5s80f0fmg3jfal".to_string()).unwrap();
        let init_msg = InstantiateMsg {
            validators: vec![(validator_pub_key.clone(), info.sender.clone())],
            chain_type: "SECRET".to_string(),
            storage_label: "storage11".to_string(),
            collection_label: "collection11".to_string(),
            collection721_code_info: CodeInfo {
                code_id: 1,
                code_hash: "87a462066a7406ff6ee66034d7c9554aae58be320f4084cb958d5b958380babb"
                    .to_string(),
            },
            storage721_code_info: CodeInfo {
                code_id: 2,
                code_hash: "87a462066a7406ff6ee66034d7c9554aae58be320f4084cb958d5b958380babb"
                    .to_string(),
            },
            collection1155_code_info: CodeInfo {
                code_id: 3,
                code_hash: "87a462066a7406ff6ee66034d7c9554aae58be320f4084cb958d5b958380babb"
                    .to_string(),
            },
            storage1155_code_info: CodeInfo {
                code_id: 4,
                code_hash: "87a462066a7406ff6ee66034d7c9554aae58be320f4084cb958d5b958380babb"
                    .to_string(),
            },
            collection_deployer_code_info: CodeInfo {
                code_id: 5,
                code_hash: "87a462066a7406ff6ee66034d7c9554aae58be320f4084cb958d5b958380babb"
                    .to_string(),
            },
            storage_deployer_code_info: CodeInfo {
                code_id: 6,
                code_hash: "87a462066a7406ff6ee66034d7c9554aae58be320f4084cb958d5b958380babb"
                    .to_string(),
            },
        };

        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

        assert_eq!(2, res.messages.len());

        let validators_count_binary =
            query(deps.as_ref(), mock_env(), QueryMsg::GetValidatorsCount {}).unwrap();

        let validator_info_binary = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetValidator {
                address: validator_pub_key,
            },
        )
        .unwrap();

        let collection_deployer_binary = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCollectionDeployer {},
        )
        .unwrap();

        let storage_deployer_binary =
            query(deps.as_ref(), mock_env(), QueryMsg::GetStorageDeployer {}).unwrap();

        let validators_count_answer = from_binary::<QueryAnswer>(&validators_count_binary).unwrap();
        let validator_answer = from_binary::<QueryAnswer>(&validator_info_binary).unwrap();
        let collection_deployer_answer =
            from_binary::<QueryAnswer>(&collection_deployer_binary).unwrap();
        let storage_deployer_answer = from_binary::<QueryAnswer>(&storage_deployer_binary).unwrap();

        match validators_count_answer {
            QueryAnswer::ValidatorCountResponse { count } => {
                assert_eq!(1, count);
            }
            _ => panic!("query error"),
        }

        match validator_answer {
            QueryAnswer::Validator { data } => {
                assert_eq!(true, data.unwrap().added);
            }
            _ => panic!("query error"),
        }

        match collection_deployer_answer {
            QueryAnswer::CollectionDeployer { data } => {
                let valid_addr = deps.api.addr_validate(&data.into_string());
                assert!(valid_addr.is_ok());
            }
            _ => panic!("query error"),
        }

        match storage_deployer_answer {
            QueryAnswer::StorageDeployer { data } => {
                let valid_addr = deps.api.addr_validate(&data.into_string());
                assert!(valid_addr.is_ok());
            }
            _ => panic!("query error"),
        }
    }

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies_with_balance(&[Coin {
    //         denom: "token".to_string(),
    //         amount: Uint128::new(2),
    //     }]);
    //     let info = mock_info(
    //         "creator",
    //         &[Coin {
    //             denom: "token".to_string(),
    //             amount: Uint128::new(2),
    //         }],
    //     );
    //     let init_msg = InstantiateMsg { count: 17 };

    //     let _res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    //     // anyone can increment
    //     let info = mock_info(
    //         "anyone",
    //         &[Coin {
    //             denom: "token".to_string(),
    //             amount: Uint128::new(2),
    //         }],
    //     );

    //     let exec_msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies_with_balance(&[Coin {
    //         denom: "token".to_string(),
    //         amount: Uint128::new(2),
    //     }]);
    //     let info = mock_info(
    //         "creator",
    //         &[Coin {
    //             denom: "token".to_string(),
    //             amount: Uint128::new(2),
    //         }],
    //     );
    //     let init_msg = InstantiateMsg { count: 17 };

    //     let _res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    //     // not anyone can reset
    //     let info = mock_info(
    //         "anyone",
    //         &[Coin {
    //             denom: "token".to_string(),
    //             amount: Uint128::new(2),
    //         }],
    //     );
    //     let exec_msg = ExecuteMsg::Reset { count: 5 };

    //     let res = execute(deps.as_mut(), mock_env(), info, exec_msg);

    //     match res {
    //         Err(StdError::GenericErr { .. }) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let info = mock_info(
    //         "creator",
    //         &[Coin {
    //             denom: "token".to_string(),
    //             amount: Uint128::new(2),
    //         }],
    //     );
    //     let exec_msg = ExecuteMsg::Reset { count: 5 };

    //     let _res = execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
