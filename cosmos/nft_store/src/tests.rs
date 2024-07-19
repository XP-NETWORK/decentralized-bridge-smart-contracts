#[cfg(test)]
mod tests {
    use std::error::Error;

    use cosmwasm_std::{Addr, Empty};
    use cw721_base::entry::query;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::{
        entry::{execute, instantiate},
        msg::{NftStoreExecuteMsg, NftStoreInstantiateMsg},
    };

    fn prepare_collection() -> ContractWrapper<
        cw721_base::ExecuteMsg<std::option::Option<cosmwasm_std::Empty>, cosmwasm_std::Empty>,
        cw721_base::InstantiateMsg,
        cw721_base::QueryMsg<cosmwasm_std::Empty>,
        cw721_base::ContractError,
        cosmwasm_std::StdError,
        cosmwasm_std::StdError,
    > {
        let code = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        code
    }

    #[test]
    fn test_contract() -> Result<(), Box<dyn Error>> {
        let mut app = App::default();

        let collection_code = prepare_collection();
        let collection_code_id = app.store_code(Box::new(collection_code));

        let nft_addr = app.instantiate_contract(
            collection_code_id,
            Addr::unchecked("owner"),
            &cw721_base::InstantiateMsg {
                minter: Addr::unchecked("minter").to_string(),
                name: "Test Contract".to_string(),
                symbol: "TEST".to_string(),
            },
            &[],
            "nft",
            None,
        )?;

        let storage_code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(storage_code));

        let addr = app.instantiate_contract(
            code_id.clone(),
            Addr::unchecked("owner"),
            &NftStoreInstantiateMsg {
                collection_address: nft_addr.clone(),
                owner: Addr::unchecked("owner"),
                collection_code_id,
                is_original: true,
                token_id: "1".to_string(),
            },
            &[],
            "Storage",
            None,
        );
        assert!(addr.is_ok());
        let addr = addr.unwrap();

        let mint = app.execute_contract(
            Addr::unchecked("minter"),
            nft_addr.clone(),
            &cw721_base::msg::ExecuteMsg::<Empty, Empty>::Mint {
                token_id: "1".to_string(),
                owner: addr.to_string(),
                token_uri: Some("https://google.com".to_string()),
                extension: Empty {},
            },
            &[],
        );
        assert!(mint.is_ok());

        let unlock = app.execute_contract(
            Addr::unchecked("owner"),
            addr,
            &NftStoreExecuteMsg::UnLockToken {
                token_id: "1".to_string(),
                to: Addr::unchecked("minter"),
            },
            &[],
        );
        assert!(unlock.is_ok());

        Ok(())
    }
}
