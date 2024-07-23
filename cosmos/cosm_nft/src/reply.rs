use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

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
