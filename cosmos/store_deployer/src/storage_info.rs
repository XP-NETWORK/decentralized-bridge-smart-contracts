use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct StorageInstantiateMsg {
    pub collection_address: Addr,
    pub owner: Addr,
    pub collection_code_id: u64,
    pub is_original: bool,
    pub token_id: String,
}
