use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct StoreFactoryInstantiateMsg {
    pub storage721_code_id: u64,
}

#[cw_serde]
pub enum StoreFactoryExecuteMsg {
    CreateStorage721 {
        label: String,
        collection_address: Addr,
        collection_code_id: u64,
        owner: String,
        is_original: bool,
        token_id: String,
    },
}

#[cw_serde]
pub enum StoreFactoryQueryMsg {}

#[cw_serde]
pub struct ReplyStorageInfo {
    pub label: String,
    pub address: Addr,
    pub is_original: bool,
    pub token_id: String,
    pub token_amount: u128,
}
