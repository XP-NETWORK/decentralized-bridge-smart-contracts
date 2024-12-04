use std::str::FromStr;

use near_sdk::{env, near, AccountId, Promise, PromiseError};

mod external;
#[near(contract_state)]
pub struct CollectionFactory {
    owner: AccountId,
}

pub const COLLECTION: &'static [u8; 331273] = include_bytes!("../../target/near/nft/nft.wasm");

impl Default for CollectionFactory {
    fn default() -> Self {
        env::panic_str("Contract should be initialized before usage")
    }
}

#[near]
impl CollectionFactory {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner }
    }

    #[payable]
    pub fn deploy_nft_collection(&mut self, name: String, symbol: String) -> Promise {
        let mut coll_str = format!(
            "{}-{}",
            convert_to_alphanumeric(&name),
            convert_to_alphanumeric(&symbol)
        );
        if coll_str.len() > 12 {
            coll_str.truncate(12);
        }
        if coll_str.chars().last().unwrap() == '-' {
            coll_str.pop();
        }
        let collection_id = AccountId::from_str(&format!(
            "{}.{}",
            coll_str,
            env::current_account_id().to_string()
        ))
        .unwrap();
        let cost = env::storage_byte_cost()
            .saturating_mul(COLLECTION.len() as u128)
            .saturating_mul(5)
            .saturating_div(4);
        let ctr = Promise::new(collection_id.clone())
            .create_account()
            .transfer(cost)
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(include_bytes!("../../target/near/nft/nft.wasm").to_vec())
            .then(external::collection::ext(collection_id.clone()).new(
                self.owner.clone(),
                external::NFTContractMetadata {
                    spec: "nep-171.0".to_string(),
                    name,
                    symbol,
                    icon: None,
                    base_uri: None,
                    reference: None,
                    reference_hash: None,
                },
            ))
            .then(Self::ext(env::current_account_id()).reply_collection_aid(collection_id));
        ctr
    }

    #[private]
    pub fn reply_collection_aid(
        &self,
        collection: AccountId,
        #[callback_result] result: Result<(), PromiseError>,
    ) -> AccountId {
        match result {
            Ok(_) => collection,
            Err(_) => {
                env::panic_str("Failed to deploy and initialize NFT collection contract");
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
        let contract = CollectionFactory::new(aid.clone());
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
