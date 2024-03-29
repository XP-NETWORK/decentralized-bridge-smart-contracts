#[cfg(test)]
mod tests {

    use collection_deployer::{
        error::CollectionFactoryContractError,
        msg::{CollectionDeployerExecuteMsg, CollectionDeployerInstantiateMsg},
    };

    use cosm_nft::{init::InstantiateMsg, royalty::RoyaltyData};
    use cosmwasm_std::{Addr, Binary, Coin};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use nft_store::{
        error::StorageContractError,
        msg::{NftStoreExecuteMsg, NftStoreInstantiateMsg, NftStoreQueryMsg},
    };

    use store_deployer::msg::{StoreFactoryExecuteMsg, StoreFactoryInstantiateMsg};

    use crate::{
        msg::{BridgeExecuteMsg, GetValidatorCountResponse},
        structs::{ClaimData, ClaimMsg, Lock721Msg, SignerAndSignature},
    };

    fn collection_code_wrapper() -> ContractWrapper<
        cosm_nft::NftExecuteMsg,
        InstantiateMsg,
        cosm_nft::NftQueryMsg,
        cosm_nft::error::ContractError,
        cosm_nft::error::ContractError,
        cosmwasm_std::StdError,
    > {
        ContractWrapper::new(
            cosm_nft::entry::execute,
            cosm_nft::entry::instantiate,
            cosm_nft::entry::query,
        )
    }

    fn store_code_wrapper() -> ContractWrapper<
        NftStoreExecuteMsg,
        NftStoreInstantiateMsg,
        NftStoreQueryMsg,
        StorageContractError,
        StorageContractError,
        StorageContractError,
    > {
        ContractWrapper::new(
            nft_store::entry::execute,
            nft_store::entry::instantiate,
            nft_store::entry::query,
        )
    }

    fn storage_factory_contracts_wrapper() -> ContractWrapper<
        StoreFactoryExecuteMsg,
        StoreFactoryInstantiateMsg,
        cosmwasm_std::Empty,
        store_deployer::error::StorageFactoryContractError,
        store_deployer::error::StorageFactoryContractError,
        store_deployer::error::StorageFactoryContractError,
        cosmwasm_std::Empty,
        cosmwasm_std::Empty,
        cosmwasm_std::Empty,
        anyhow::Error,
        store_deployer::error::StorageFactoryContractError,
    > {
        let storage_factory_code = ContractWrapper::new(
            store_deployer::entry::execute,
            store_deployer::entry::instantiate,
            store_deployer::entry::query,
        );
        storage_factory_code.with_reply(store_deployer::entry::reply)
    }

    fn collection_factory_contracts_wrapper() -> ContractWrapper<
        CollectionDeployerExecuteMsg,
        CollectionDeployerInstantiateMsg,
        cosmwasm_std::Empty,
        CollectionFactoryContractError,
        CollectionFactoryContractError,
        CollectionFactoryContractError,
        cosmwasm_std::Empty,
        cosmwasm_std::Empty,
        cosmwasm_std::Empty,
        anyhow::Error,
        CollectionFactoryContractError,
    > {
        let collection_factory_code = ContractWrapper::new(
            collection_deployer::entry::execute,
            collection_deployer::entry::instantiate,
            collection_deployer::entry::query,
        );
        let collection_factory_code =
            collection_factory_code.with_reply(collection_deployer::entry::reply);
        collection_factory_code
    }

    fn save_factory_contracts(app: &mut App) -> (u64, u64) {
        let (sfc, cfc) = (
            storage_factory_contracts_wrapper(),
            collection_factory_contracts_wrapper(),
        );
        let sfc_id = app.store_code(Box::new(sfc));
        let cfc_id = app.store_code(Box::new(cfc));
        return (sfc_id, cfc_id);
    }

    fn save_collection_and_store_contracts(app: &mut App) -> (u64, u64) {
        let collection_code_id = app.store_code(Box::new(collection_code_wrapper()));
        let store_code_id = app.store_code(Box::new(store_code_wrapper()));
        return (collection_code_id, store_code_id);
    }

    #[test]
    fn initializes_correctly() {
        let mut app = App::default();
        let deployer = Addr::unchecked("deployer");

        let (sfc_id, cfc_id) = save_factory_contracts(&mut app);
        let (collection_code_id, storage721_code_id) =
            save_collection_and_store_contracts(&mut app);

        let bridge_contract_wrapper = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);

        let bridge_code_id = app.store_code(Box::new(bridge_contract_wrapper));

        let msg = crate::structs::BridgeInstantiateMsg {
            validators: vec![],
            chain_type: "cosmos".to_string(),
            storage_label: "xp-nft-store-factory".to_string(),
            collection_label: "xp-nft-collection-factory".to_string(),
            collection721_code_id: collection_code_id,
            storage721_code_id,
            collection_deployer_code_id: cfc_id,
            storage_deployer_code_id: sfc_id,
        };

        let bridge_instantiate = app.instantiate_contract(
            bridge_code_id,
            deployer,
            &msg,
            &[],
            "xp_bridge".to_string(),
            None,
        );
        assert!(bridge_instantiate.is_ok(), "{:?}", bridge_instantiate);
        let bridge_addr = bridge_instantiate.unwrap();

        let vc = app.wrap().query_wasm_smart::<GetValidatorCountResponse>(
            bridge_addr,
            &crate::msg::BridgeQueryMsg::GetValidatorsCount {},
        );
        assert!(vc.is_ok());
        assert!(vc.unwrap().count == 0, "Expected 0 validators");
    }

    #[test]
    fn test_lock_nft() {
        let mut app = App::default();
        let deployer = Addr::unchecked("deployer");
        let nft_minter = Addr::unchecked("nft_minter");

        let (sfc_id, cfc_id) = save_factory_contracts(&mut app);
        let (collection_code_id, storage721_code_id) =
            save_collection_and_store_contracts(&mut app);

        let bridge_contract_wrapper = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);

        let bridge_code_id = app.store_code(Box::new(bridge_contract_wrapper));

        let msg = crate::structs::BridgeInstantiateMsg {
            validators: vec![],
            chain_type: "cosmos".to_string(),
            storage_label: "xp-nft-store-factory".to_string(),
            collection_label: "xp-nft-collection-factory".to_string(),
            collection721_code_id: collection_code_id,
            storage721_code_id,
            collection_deployer_code_id: cfc_id,
            storage_deployer_code_id: sfc_id,
        };

        let bridge_instantiate = app.instantiate_contract(
            bridge_code_id,
            deployer.clone(),
            &msg,
            &[],
            "xp_bridge".to_string(),
            None,
        );
        assert!(bridge_instantiate.is_ok(), "{:?}", bridge_instantiate);
        let bridge_addr = bridge_instantiate.unwrap();

        let nft_contract = app.instantiate_contract(
            collection_code_id,
            deployer.clone(),
            &cosm_nft::init::InstantiateMsg {
                name: "test-nft-contract".to_string(),
                symbol: "TNC".to_string(),
                minter: nft_minter.clone().to_string(),
                destination_user_address: Addr::unchecked("input"),
                metadata: Default::default(),
                royalty: Default::default(),
                source_chain: Default::default(),
                source_nft_contract_address: Default::default(),
                token_id: Default::default(),
                royalty_receiver: Addr::unchecked("rr"),
                token_amount: Default::default(),
            },
            &[],
            "test-nft-contract",
            None,
        );
        assert!(nft_contract.is_ok(), "nft contract failed to instantiate");
        let nft_contract = nft_contract.unwrap();

        let mint = app.execute_contract(
            nft_minter.clone(),
            nft_contract.clone(),
            &cosm_nft::NftExecuteMsg::Mint {
                token_id: "1".to_owned(),
                owner: nft_minter.clone().into_string(),
                token_uri: Some("https://example.com".to_owned()),
                extension: RoyaltyData {
                    ..Default::default()
                },
            },
            &[],
        );
        let approve = app.execute_contract(
            nft_minter.clone(),
            nft_contract.clone(),
            &cosm_nft::NftExecuteMsg::Approve {
                spender: bridge_addr.to_string(),
                token_id: "1".to_string(),
                expires: None,
            },
            &[],
        );
        assert!(approve.is_ok(), "approve failed: {:?}", approve);
        assert!(mint.is_ok(), "mint failed: {:?}", mint);
        let lock = app.execute_contract(
            deployer.clone(),
            bridge_addr,
            &crate::msg::BridgeExecuteMsg::Lock721 {
                data: Lock721Msg {
                    collection_code_id,
                    destination_chain: "BSC".to_string(),
                    destination_user_address: "0xabc123".to_string(),
                    source_nft_contract_address: nft_contract.clone(),
                    token_id: "1".to_string(),
                },
            },
            &[],
        );
        assert!(lock.is_ok(), "lock failed: {:?}", lock);
    }

    #[test]
    fn test_claim_nft() {
        use secp256k1::rand::rngs::OsRng;
        use secp256k1::Message;
        use sha2::Digest;
        let mut app = App::default();
        let deployer = Addr::unchecked("deployer");

        let (sfc_id, cfc_id) = save_factory_contracts(&mut app);
        let (collection_code_id, storage721_code_id) =
            save_collection_and_store_contracts(&mut app);

        let bridge_contract_wrapper = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);

        let bridge_code_id = app.store_code(Box::new(bridge_contract_wrapper));
        let secp = secp256k1::Secp256k1::new();
        let (sk, pubk) = secp.generate_keypair(&mut OsRng);
        let public_key = Binary::from(pubk.serialize());

        let msg = crate::structs::BridgeInstantiateMsg {
            validators: vec![(public_key.clone(), Addr::unchecked("validator1"))],
            chain_type: "cosmos".to_string(),
            storage_label: "xp-nft-store-factory".to_string(),
            collection_label: "xp-nft-collection-factory".to_string(),
            collection721_code_id: collection_code_id,
            storage721_code_id,
            collection_deployer_code_id: cfc_id,
            storage_deployer_code_id: sfc_id,
        };

        let bridge_instantiate = app.instantiate_contract(
            bridge_code_id,
            deployer.clone(),
            &msg,
            &[],
            "xp_bridge".to_string(),
            None,
        );
        assert!(bridge_instantiate.is_ok(), "{:?}", bridge_instantiate);
        let bridge_addr = bridge_instantiate.unwrap();

        let cd = ClaimData {
            destination_chain: "cosmos".to_string(),
            destination_user_address: Addr::unchecked("claimer"),
            fee: 1000,
            metadata: "metadata".to_string(),
            name: "name".to_string(),
            nft_type: "singular".to_string(),
            royalty: 1,
            royalty_receiver: Addr::unchecked("royalty_receiver"),
            source_chain: "bsc".to_string(),
            source_nft_contract_address: "bruh".to_string(),
            token_id: "1".to_string(),
            symbol: "BRUH".to_string(),
            token_amount: 1,
            transaction_hash: "0xabc123".to_string(),
        };

        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: "deployer".to_string(),
                amount: vec![Coin::new(1000, "uscrt")],
            },
        ))
        .unwrap();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&cd.concat_all_fields());
        let data: [u8; 32] = hasher.finalize().into();
        let msg = Message::from_digest(data);
        println!("{:x?}", msg);

        let signed = Binary::from(secp.sign_ecdsa(&msg, &sk).serialize_compact());

        let claim = app.execute_contract(
            deployer,
            bridge_addr,
            &BridgeExecuteMsg::Claim721 {
                data: ClaimMsg {
                    data: cd,
                    signatures: vec![SignerAndSignature {
                        signature: signed,
                        signer_address: public_key,
                    }],
                },
            },
            &[Coin::new(1000, "uscrt")],
        );
        assert!(claim.is_ok(), "claim failed: {:?}", claim);
    }
}
