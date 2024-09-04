use near_sdk::{ext_contract, Promise};

#[allow(dead_code)]
#[ext_contract(ext_nft)]
pub trait NFT {
    fn nft_transfer(
        &mut self,
        receiver_id: near_sdk::AccountId,
        token_id: nft::TokenId,
        //we introduce an approval ID so that people with that approval ID can transfer the token
        approval_id: Option<u32>,
        memo: Option<String>,
    );
}


#[allow(dead_code)]
#[ext_contract(storage_factory)]
pub trait StorageFactory {
    fn deploy_nft_storage(&mut self, collection: near_sdk::AccountId) -> Promise;
}

#[allow(dead_code)]
#[ext_contract(collection_factory)]
pub trait CollectionFactory {
    fn deploy_nft_collection(&mut self, name: String, symbol: String) -> Promise;
}