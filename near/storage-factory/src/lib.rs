use std::str::FromStr;

use near_sdk::{env, near, AccountId, Promise, PromiseError};

mod external;
#[near(contract_state)]
pub struct StorageFactory {
    owner: AccountId,
}

impl Default for StorageFactory {
    fn default() -> Self {
        env::panic_str("Contract should be initialized before usage")
    }
}
const STORAGE: &'static [u8; 122437] = include_bytes!("../../target/near/storage/storage.wasm");

#[near]
impl StorageFactory {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner }
    }

    #[payable]
    pub fn deploy_nft_storage(&mut self, collection: AccountId) -> Promise {
        let coll_str = collection.to_string();
        let mut collection_shortened = coll_str.split(".").next().unwrap();
        if collection_shortened.len() > 12 {
            collection_shortened = &collection_shortened[..12];
        }
        let aid = AccountId::from_str(&format!(
            "{}.{}",
            convert_to_alphanumeric(collection_shortened),
            env::current_account_id().to_string()
        ))
        .unwrap();
        let cost = env::storage_byte_cost()
            .saturating_mul(STORAGE.len() as u128)
            .saturating_mul(5)
            .saturating_div(4);
        let ctr = Promise::new(aid.clone())
            .create_account()
            .transfer(cost) // 5e24yN, 5N9
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(STORAGE.to_vec())
            .then(external::storage::ext(aid.clone()).new(env::current_account_id(), collection))
            .then(Self::ext(env::current_account_id()).reply_storage_aid(aid));
        ctr
    }

    pub fn reply_storage_aid(
        &self,
        storage: AccountId,
        #[callback_result] result: Result<(), PromiseError>,
    ) -> AccountId {
        match result {
            Ok(_) => storage,
            Err(_) => {
                env::panic_str("Failed to deploy and initialize NFT storage contract");
            }
        }
    }

    pub fn owner(&self) -> AccountId {
        self.owner.clone()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn initializes_correctly() {
        let aid = AccountId::from_str("aid").unwrap();
        let contract = StorageFactory::new(aid.clone());
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.owner(), aid);
    }
}

fn convert_to_alphanumeric(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
}