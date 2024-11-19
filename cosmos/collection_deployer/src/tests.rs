#[cfg(test)]
mod tests {
    use std::error::Error;

    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Empty};
    use cw721_base::entry::query;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::{
        entry::{execute, instantiate, reply},
        msg::{CollectionDeployerExecuteMsg, CollectionDeployerInstantiateMsg},
    };

    fn prepare_collection() -> ContractWrapper<
        cw721_base::ExecuteMsg<cosm_nft::royalty::RoyaltyData, cosmwasm_std::Empty>,
        cosm_nft::init::InstantiateMsg,
        cw721_base::QueryMsg<cosm_nft::msg::CW2981QueryMsg>,
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

    #[test]
    fn test_contract() -> Result<(), Box<dyn Error>> {
        let mut app = App::default();

        let cc_code = prepare_collection();
        let cc_code_id = app.store_code(Box::new(cc_code));

        let cf_code = ContractWrapper::new(execute, instantiate, query);
        let cf_code = cf_code.with_reply(reply);
        let cf_id = app.store_code(Box::new(cf_code));

        let addr = app.instantiate_contract(
            cf_id.clone(),
            Addr::unchecked("owner"),
            &CollectionDeployerInstantiateMsg {
                collection721_code_id: cc_code_id,
            },
            &[],
            "Collection Factory",
            None,
        );
        assert!(addr.is_ok());
        let addr = addr.unwrap();

        let create = app.execute_contract(
            Addr::unchecked("owner"),
            addr,
            &CollectionDeployerExecuteMsg::CreateCollection721 {
                owner: Addr::unchecked("owner").to_string(),
                name: "Test Collection".to_string(),
                symbol: "TEST".to_string(),
                source_nft_contract_address: "".to_string(),
                source_chain: "".to_string(),
                destination_user_address: Addr::unchecked("receiver"),
                token_id: "".to_string(),
                token_amount: 1,
                royalty: 1,
                royalty_receiver: Addr::unchecked("receiver"),
                metadata: "".to_string(),
                transaction_hash: "transaction_hash".to_string(),
                lock_tx_chain: "BRUH".to_string()
            },
            &[],
        );

        assert!(create.is_ok());
        let create = create.unwrap();
        let created = create.events.iter().find(|e| e.ty == "instantiate");
        let addr = created
            .unwrap()
            .attributes
            .iter()
            .find(|a| a.key == "_contract_addr")
            .map(|attr| attr.value.clone());
        assert!(addr.is_some());
        let addr = addr.unwrap();

        #[cw_serde]
        pub struct ContractInfoResponse {
            pub name: String,
            pub symbol: String,
        }

        let result: Result<ContractInfoResponse, _> = app.wrap().query_wasm_smart(
            Addr::unchecked(addr),
            &cw721_base::QueryMsg::<Empty>::ContractInfo {},
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.name, "Test Collection");
        assert_eq!(result.symbol, "TEST");

        Ok(())
    }
}
