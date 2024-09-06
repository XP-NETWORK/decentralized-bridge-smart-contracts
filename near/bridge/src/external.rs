use std::collections::HashMap;

use near_sdk::{ext_contract, AccountId, Promise};
use self::nft_types::{JsonToken, TokenId, TokenMetadata};

#[allow(dead_code)]
#[ext_contract(ext_nft)]
pub trait NFT {
    fn nft_transfer(
        &mut self,
        receiver_id: near_sdk::AccountId,
        token_id: self::nft_types::TokenId,
        //we introduce an approval ID so that people with that approval ID can transfer the token
        approval_id: Option<u32>,
        memo: Option<String>,
    );

    fn nft_tokens_for_owner(
        &self,
        account_id: near_sdk::AccountId,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u32>,
    ) -> Vec<JsonToken>;

    fn nft_mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        //we add an optional parameter for perpetual royalties
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
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

pub mod nft_types {
    use std::collections::HashMap;

    use near_sdk::{
        borsh::{BorshDeserialize, BorshSerialize},
        json_types::Base64VecU8,
        serde::{Deserialize, Serialize},
        AccountId, NearSchema,
    };

    #[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
    #[borsh(crate = "near_sdk::borsh")]
    #[serde(crate = "near_sdk::serde")]
    pub struct TokenMetadata {
        pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
        pub description: Option<String>, // free-form description
        pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
        pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
        pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
        pub issued_at: Option<u64>, // When token was issued or minted, Unix epoch in milliseconds
        pub expires_at: Option<u64>, // When token expires, Unix epoch in milliseconds
        pub starts_at: Option<u64>, // When token starts being valid, Unix epoch in milliseconds
        pub updated_at: Option<u64>, // When token was last updated, Unix epoch in milliseconds
        pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
        pub reference: Option<String>, // URL to an off-chain JSON file with more info.
        pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
    }

    pub type TokenId = String;

    //The Json token is what will be returned from view calls.
    #[derive(Serialize, Deserialize, NearSchema)]
    #[serde(crate = "near_sdk::serde")]
    pub struct JsonToken {
        //token ID
        pub token_id: TokenId,
        //owner of the token
        pub owner_id: AccountId,
        //token metadata
        pub metadata: TokenMetadata,
        //list of approved account IDs that have access to transfer the token. This maps an account ID to an approval ID
        pub approved_account_ids: HashMap<AccountId, u32>,
        //keep track of the royalty percentages for the token in a hash map
        pub royalty: HashMap<AccountId, u32>,
    }
}

#[allow(dead_code)]
#[ext_contract(nft_storage)]
pub trait NFTStorage {
    fn unlock_token(&mut self, to: AccountId, token_id: String) -> Promise;
}
