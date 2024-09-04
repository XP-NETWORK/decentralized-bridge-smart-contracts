use std::collections::HashMap;

use near_sdk::{ext_contract, AccountId, Promise};
use nft::{JsonToken, TokenId, TokenMetadata};

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

    fn nft_tokens_for_owner(
        &self,
        account_id: near_sdk::AccountId,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u32>,
    ) -> Vec<JsonToken> ;

    fn nft_mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        //we add an optional parameter for perpetual royalties
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) ;
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




#[allow(dead_code)]
#[ext_contract(nft_storage)]
pub trait NFTStorage {
    fn unlock_token(&mut self, to: AccountId, token_id: String) -> Promise ;
}
