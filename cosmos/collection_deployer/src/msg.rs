use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

/// Instantiation message
#[cw_serde]
pub struct CollectionDeployerInstantiateMsg {
    /// collection code info
    pub collection721_code_id: u64,
}

#[cw_serde]
pub enum CollectionDeployerExecuteMsg {
    CreateCollection721 {
        owner: String,
        name: String,
        symbol: String,
        source_nft_contract_address: String,
        source_chain: String,
        destination_user_address: Addr,
        token_id: String,
        token_amount: u128,
        royalty: u16,
        royalty_receiver: Addr,
        metadata: String,
        transaction_hash: String,
        lock_tx_chain: String
    },
}

#[cw_serde]
pub enum CollectionDeployerQueryMsg {}

#[cw_serde]
pub struct ReplyCollectionInfo {
    pub label: String,
    pub owner: Addr,
    pub address: Addr,
    pub source_nft_contract_address: String,
    pub source_chain: String,
    pub destination_user_address: Addr,
    pub token_id: String,
    pub token_amount: u128,
    pub royalty: u16,
    pub royalty_receiver: Addr,
    pub metadata: String,
    pub transaction_hash: String,
    pub lock_tx_chain: String
}
