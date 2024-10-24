use std::str::FromStr;

use near_sdk::{env, near, AccountId, NearToken, Promise, PromiseError};

mod external;
#[near(contract_state)]
pub struct CollectionFactory {
    owner: AccountId,
}

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
        env::log_str(&format!(
            "{}-{}.{}",
            name.split_whitespace().collect::<String>().to_lowercase(),
            symbol.to_lowercase(),
            env::current_account_id().to_string()
        ));
        let collection_id = AccountId::from_str(&format!(
            "{}-{}.{}",
            name.split_whitespace().collect::<String>().to_lowercase(),
            symbol.to_lowercase(),
            env::current_account_id().to_string()
        ))
        .unwrap();
        let ctr = Promise::new(collection_id.clone())
            .create_account()
            .transfer(NearToken::from_near(5)) // 5e24yN, 5N
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(
                include_bytes!("../../target/near/nft/nft.wasm").to_vec(),
            )
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
