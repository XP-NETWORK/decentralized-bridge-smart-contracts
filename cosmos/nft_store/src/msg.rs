use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct NftStoreInstantiateMsg {
    pub collection_address: Addr,
    pub owner: Addr,
    pub collection_code_id: u64,
    pub is_original: bool,
    pub token_id: String,
}

#[cw_serde]
pub enum NftStoreExecuteMsg {
    UnLockToken { token_id: String, to: Addr },
}

#[cw_serde]
pub enum NftStoreQueryMsg {
    GetCollectionAddress,
}
