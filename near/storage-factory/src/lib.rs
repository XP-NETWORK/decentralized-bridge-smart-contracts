use std::str::FromStr;

use near_sdk::{env, near, AccountId, NearToken, Promise};

mod external;
#[near(contract_state)]
pub struct StorageFactory {
    owner: AccountId,
}

impl Default for StorageFactory {
    fn default() -> Self {
        Self {
            owner: "default.near".parse().unwrap(),
        }
    }
}

#[near]
impl StorageFactory {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner }
    }

    #[payable]
    pub fn deploy_nft_storage(&mut self, collection: AccountId) -> Promise {
        let aid = AccountId::from_str(&format!(
            "{collection}{}",
            env::current_account_id().to_string()
        ))
        .unwrap();
        let ctr = Promise::new(aid.clone())
            .create_account()
            .transfer(NearToken::from_near(5)) // 5e24yN, 5N
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(
                include_bytes!("../../target/wasm32-unknown-unknown/release/storage.wasm").to_vec(),
            )
            .then(external::storage::ext(aid.clone()).new(env::current_account_id(), collection))
            .then(Self::ext(env::current_account_id()).reply_storage_aid(aid));
        ctr
    }

    pub fn reply_storage_aid(
        &self,
        storage: AccountId,
        #[callback] result: Result<(), Promise>,
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
