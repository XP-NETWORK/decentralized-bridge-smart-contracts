use ed25519_dalek::{PublicKey, Verifier};
use events::{EventLog, EventLogVariant, NewValidatorAdded, ValidatorBlacklisted};
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    collections::{LookupMap, TreeMap},
    env, near, require,
    serde::{Deserialize, Serialize},
    AccountId, NearSchema, NearToken, Promise, PromiseError,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct ContractInfo {
    chain: String,
    contract_address: String,
}

use nft::TokenId;
use serde_json::to_string;
mod external;
#[near(contract_state)]
pub struct Bridge {
    collection_factory: AccountId,
    storage_factory: AccountId,
    validators: TreeMap<String, u128>,
    blacklisted_validators: LookupMap<String, u128>,
    self_chain: String,
    duplicate_to_original_mapping: LookupMap<(AccountId, String), ContractInfo>,
    original_storage_mapping: LookupMap<(String, String), AccountId>,
    duplicate_storage_mapping: LookupMap<(String, String), AccountId>,
}

mod events;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct SignerAndSignature {
    signer: Vec<u8>,
    signature: Vec<u8>,
}

impl Default for Bridge {
    #![allow(unreachable_code)]
    fn default() -> Self {
        Self {
            collection_factory: todo!(),
            storage_factory: todo!(),
            validators: todo!(),
            blacklisted_validators: todo!(),
            ..Default::default()
        }
    }
}

#[near]
impl Bridge {
    #[init]
    pub fn new(
        collection_factory: AccountId,
        storage_factory: AccountId,
        validators: Vec<String>,
    ) -> Self {
        let mut v = TreeMap::new(b"v");
        for validator in validators {
            v.insert(&validator, &0);
        }
        Self {
            validators: v,
            collection_factory,
            storage_factory,
            blacklisted_validators: LookupMap::new(b"b"),
            self_chain: "near".to_string(),
            duplicate_to_original_mapping: LookupMap::new(b"d"),
            original_storage_mapping: LookupMap::new(b"o"),
            duplicate_storage_mapping: LookupMap::new(b"a"),
        }
    }

    pub fn validator_count(&self) -> u128 {
        self.validators.len() as u128
    }

    // #[payable]
    // pub fn deploy_nft_collection(&mut self, name: String, symbol: String) -> Promise {
    //     let collection_id = AccountId::from_str(&format!(
    //         "{name}-{symbol}.{}",
    //         env::current_account_id().to_string()
    //     ))
    //     .unwrap();
    //     let ctr = Promise::new(collection_id.clone())
    //         .create_account()
    //         .transfer(NearToken::from_near(5)) // 5e24yN, 5N
    //         .add_full_access_key(env::signer_account_pk())
    //         .deploy_contract(
    //             include_bytes!("../../target/wasm32-unknown-unknown/release/storage.wasm").to_vec(),
    //         )
    //         .then(external::collection::ext(collection_id).new(
    //             env::current_account_id(),
    //             nft::NFTContractMetadata {
    //                 spec: "nep-171.0".to_string(),
    //                 name,
    //                 symbol,
    //                 icon: None,
    //                 base_uri: None,
    //                 reference: None,
    //                 reference_hash: None,
    //             },
    //         ));
    //     ctr
    // }

    pub fn verify_signatures(
        &mut self,
        message: Vec<u8>,
        signer_and_signature: Vec<SignerAndSignature>,
    ) -> Vec<Vec<u8>> {
        let mut percentage = 0;
        let mut validators_to_reward = Vec::new();

        for ss in signer_and_signature {
            let signer = PublicKey::from_bytes(&ss.signer).unwrap();
            let signature = ed25519_dalek::Signature::from_bytes(&ss.signature).unwrap();
            let valid = signer
                .verify(&message, &signature)
                .map(|_| true)
                .unwrap_or(false);
            if valid {
                percentage += 1;
                validators_to_reward.push(ss.signature);
            }
        }
        require!(percentage >= self.threshold(), "Insufficient signatures");
        validators_to_reward
    }

    pub fn add_validator(&mut self, validator: String, signatures: Vec<SignerAndSignature>) {
        require!(
            !self.blacklisted_validators.contains_key(&validator),
            "Validator is blacklisted"
        );
        require!(
            self.validators.contains_key(&validator),
            "Validator already exists"
        );
        self.verify_signatures(validator.clone().into_bytes(), signatures);
        env::log_str(
            &to_string(&EventLog {
                standard: "bridge".to_string(),
                version: "1.0.0".to_string(),
                event: EventLogVariant::ValidatorAdded(NewValidatorAdded { validator }),
            })
            .unwrap(),
        );
    }

    pub fn lock_nft(
        &mut self,
        source_nft_contract_address: AccountId,
        token_id: TokenId,
        destination_chain: String,
        destination_address: String,
    ) {
        require!(
            destination_chain != self.self_chain,
            "Destination chain is the same as source chain"
        );
        let oca = self
            .duplicate_to_original_mapping
            .get(&(source_nft_contract_address.clone(), self.self_chain.clone()));

        match oca {
            Some(_) => {
                Self::check_storage_nft(
                    self.self_chain.clone(),
                    source_nft_contract_address,
                    token_id,
                    destination_chain,
                    destination_address,
                    &mut self.duplicate_storage_mapping,
                    self.storage_factory.clone(),
                );
            }
            None => {
                Self::check_storage_nft(
                    self.self_chain.clone(),
                    source_nft_contract_address,
                    token_id,
                    destination_chain,
                    destination_address,
                    &mut self.original_storage_mapping,
                    self.storage_factory.clone(),
                );
            }
        }
    }

    fn check_storage_nft(
        self_chain: String,
        source_nft_contract_address: AccountId,
        token_id: TokenId,
        destination_chain: String,
        destination_address: String,
        storage: &mut LookupMap<(String, String), AccountId>,
        sf: AccountId,
    ) -> Promise {
        let storage_address_opt =
            storage.get(&(source_nft_contract_address.to_string(), self_chain.clone()));

        match storage_address_opt {
            Some(storage_address) => external::ext_nft::ext(source_nft_contract_address.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .nft_transfer(storage_address, token_id.clone(), None, None)
                .then(Self::ext(env::current_account_id()).emit_locked_event(
                    destination_chain,
                    destination_address,
                    source_nft_contract_address,
                    token_id,
                )),
            None => external::storage_factory::ext(sf)
                .deploy_nft_storage(source_nft_contract_address.clone())
                .then(Self::ext(env::current_account_id()).transfer_to_storage(
                    source_nft_contract_address,
                    token_id,
                    destination_chain,
                    destination_address,
                )),
        }
    }

    pub fn transfer_to_storage(
        &mut self,
        source_nft_contract_address: AccountId,
        token_id: TokenId,
        destination_chain: String,
        destination_user_address: String,
        #[callback] result: Result<AccountId, PromiseError>,
    ) -> Promise {
        match result {
            Ok(storage_address) => external::ext_nft::ext(source_nft_contract_address.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .nft_transfer(storage_address, token_id.clone(), None, None)
                .then(Self::ext(env::current_account_id()).emit_locked_event(
                    destination_chain,
                    destination_user_address,
                    source_nft_contract_address,
                    token_id,
                )),
            Err(e) => env::panic_str(&format!("Failed to deploy Collection: {:?}", e)),
        }
    }

    #[private]
    pub fn emit_locked_event(
        &mut self,
        destination_chain: String,
        destination_address: String,
        source_nft_contract_address: AccountId,
        token_id: TokenId,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        require!(result.is_ok(), "NFT transfer failed");
        self.emit_event(EventLogVariant::Locked(events::LockedEvent {
            destination_chain,
            destination_user_address: destination_address,
            source_nft_contract_address: source_nft_contract_address.to_string(),
            token_id,
            nft_type: "singular".to_string(),
            source_chain: self.self_chain.clone(),
            token_amount: 1,
        }));
    }

    pub fn blacklist_validator(&mut self, validator: String, signatures: Vec<SignerAndSignature>) {
        require!(
            !self.blacklisted_validators.contains_key(&validator),
            "Validator is already blacklisted"
        );
        self.verify_signatures(validator.clone().into_bytes(), signatures);
        self.validators.remove(&validator);
        self.emit_event(EventLogVariant::ValidatorBlacklisted(
            ValidatorBlacklisted { validator },
        ));
    }

    pub fn threshold(&self) -> u128 {
        ((self.validators.len() * 2) / 3) as u128 + 1
    }

    fn emit_event(&self, event: EventLogVariant) {
        env::log_str(
            &to_string(&EventLog {
                standard: "xp-bridge".to_string(),
                version: "1.0.0".to_string(),
                event,
            })
            .unwrap(),
        );
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
        let cid = AccountId::from_str("aid").unwrap();
        let sid = AccountId::from_str("aid").unwrap();
        Bridge::new(cid, sid, vec![]);
    }
}
