use std::str::FromStr;

use near_sdk::{env, near, AccountId, NearToken, Promise};

mod external;
#[near(contract_state)]
pub struct CollectionFactory {
    owner: AccountId,
}

impl Default for CollectionFactory {
    fn default() -> Self {
        Self {
            owner: "default.near".parse().unwrap(),
        }
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
        let collection_id = AccountId::from_str(&format!(
            "{name}-{symbol}.{}",
            env::current_account_id().to_string()
        ))
        .unwrap();
        let ctr = Promise::new(collection_id.clone())
            .create_account()
            .transfer(NearToken::from_near(5)) // 5e24yN, 5N
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(
                include_bytes!("../../target/wasm32-unknown-unknown/release/storage.wasm").to_vec(),
            )
            .then(external::collection::ext(collection_id).new(
                env::current_account_id(),
                nft::NFTContractMetadata {
                    spec: "nep-171.0".to_string(),
                    name,
                    symbol,
                    icon: None,
                    base_uri: None,
                    reference: None,
                    reference_hash: None,
                },
            ));
        ctr
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
