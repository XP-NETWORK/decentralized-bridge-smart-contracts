use near_sdk::{near, AccountId, NearToken, Promise};

mod external;
#[near(contract_state)]
pub struct Contract {
    owner: AccountId,
    collection: AccountId,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            owner: "default.near".parse().unwrap(),
            collection: "default.near".parse().unwrap(),
        }
    }
}

#[near]
impl Contract {
    #[init]
    pub fn new(owner: AccountId, collection: AccountId) -> Self {
        Self { owner, collection }
    }

    #[payable]
    pub fn unlock_token(&mut self, to: AccountId, token_id: String) -> Promise {
        let ctr = external::common_nft::ext(self.collection.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1));
        let transfer = ctr.nft_transfer(to, token_id, None, None);
        return transfer;
    }

    pub fn owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn collection(&self) -> AccountId {
        self.collection.clone()
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
        let collection = AccountId::from_str("collection").unwrap();
        let contract = Contract::new(aid.clone(), collection.clone());
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.owner(), aid);
        assert_eq!(contract.collection(), collection);
    }
}
