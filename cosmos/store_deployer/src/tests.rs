#[cfg(test)]
mod tests {
    use std::error::Error;

    use cosmwasm_std::Addr;
    use cw721_base::entry::query;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::{
        entry::{execute, instantiate, reply},
        msg::{StoreFactoryExecuteMsg, StoreFactoryInstantiateMsg},
    };

    fn prepare_storage() -> ContractWrapper<
        nft_store::msg::NftStoreExecuteMsg,
        nft_store::msg::NftStoreInstantiateMsg,
        nft_store::msg::NftStoreQueryMsg,
        nft_store::error::StorageContractError,
        nft_store::error::StorageContractError,
        nft_store::error::StorageContractError,
    > {
        let code = ContractWrapper::new(
            nft_store::entry::execute,
            nft_store::entry::instantiate,
            nft_store::entry::query,
        );
        code
    }

    #[test]
    fn test_contract() -> Result<(), Box<dyn Error>> {
        let mut app = App::default();

        let storage_code = prepare_storage();

        let storage_code_id = app.store_code(Box::new(storage_code));

        let storage_factory_code = ContractWrapper::new(execute, instantiate, query);
        let storage_factory_code = storage_factory_code.with_reply(reply);
        let storage_factory_code_id: u64 = app.store_code(Box::new(storage_factory_code));

        let addr = app.instantiate_contract(
            storage_factory_code_id.clone(),
            Addr::unchecked("owner"),
            &StoreFactoryInstantiateMsg {
                storage721_code_id: storage_code_id,
            },
            &[],
            "Storage",
            None,
        );
        assert!(addr.is_ok());
        let addr = addr.unwrap();

        let create = app.execute_contract(
            Addr::unchecked("owner"),
            addr,
            &StoreFactoryExecuteMsg::CreateStorage721 {
                label: "nft".to_string(),
                collection_address: Addr::unchecked("collection"),
                collection_code_id: 1231,
                owner: "owner".to_string(),
                is_original: true,
                token_id: "1".to_string(),
            },
            &[],
        );
        assert!(create.is_ok());
        let create = create.unwrap();
        let inst_ev = create
            .events
            .iter()
            .find(|ev| ev.ty == "instantiate".to_string());

        assert!(inst_ev.is_some());
        let inst_ev = inst_ev.unwrap().clone();
        let address = inst_ev
            .attributes
            .iter()
            .find(|e| e.key == "_contract_addr");
        assert!(address.is_some());
        let address = address.unwrap().value.clone();

        let a: Addr = app
            .wrap()
            .query_wasm_smart(
                address,
                &nft_store::msg::NftStoreQueryMsg::GetCollectionAddress,
            )
            .unwrap();
        assert_eq!(a, Addr::unchecked("collection"));

        Ok(())
    }
}
