use std::{collections::HashMap, str::FromStr};

use ed25519_dalek::{PublicKey, Verifier};
use events::{EventLog, EventLogVariant, NewValidatorAdded, ValidatorBlacklisted};
use near_sdk::{
    collections::{LookupMap, TreeMap},
    env::{self, sha256},
    near, require, AccountId, NearToken, Promise, PromiseError,
};

#[cfg(test)]
mod tests;

mod types;

use nft::{JsonToken, TokenId, TokenMetadata};
use serde_json::to_string;
use types::{ClaimData, ContractInfo, SignerAndSignature};
mod external;
#[near(contract_state)]
pub struct Bridge {
    collection_factory: AccountId,
    storage_factory: AccountId,
    validators: TreeMap<String, u128>,
    blacklisted_validators: LookupMap<String, u128>,
    self_chain: String,
    duplicate_to_original_mapping: LookupMap<(AccountId, String), ContractInfo>,
    original_to_duplicate_mapping: LookupMap<(String, String), ContractInfo>,
    original_storage_mapping: LookupMap<(String, String), AccountId>,
    duplicate_storage_mapping: LookupMap<(String, String), AccountId>,
    unique_identifiers: LookupMap<String, bool>,
}

mod events;

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
            unique_identifiers: LookupMap::new(b"u"),
            original_to_duplicate_mapping: LookupMap::new(b"r"),
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

    pub fn claim_nft(&mut self, cd: ClaimData, sigs: Vec<SignerAndSignature>) -> Promise {
        assert!(
            env::attached_deposit() > NearToken::from_yoctonear(cd.fee),
            "Insufficient fee"
        );
        assert!(
            cd.destination_chain == self.self_chain,
            "Invalid destination chain"
        );
        assert!(cd.nft_type == "singular", "Invalid NFT type");
        let serialized = near_sdk::borsh::to_vec(&cd).unwrap();
        let hash = sha256(&serialized);
        let hexeh = hex::encode(hash);
        require!(
            self.unique_identifiers.get(&hexeh).is_none(),
            "Data already processed!"
        );
        let validators_to_reward = self.verify_signatures(serialized, sigs);
        self.reward_validators(cd.fee, validators_to_reward);

        let duplicate_collection_address = self.original_to_duplicate_mapping.get(&(
            cd.source_nft_contract_address.clone(),
            cd.source_chain.clone(),
        ));

        let has_duplicate = duplicate_collection_address.is_some();
        let storage = if has_duplicate {
            self.duplicate_storage_mapping.get(&(
                duplicate_collection_address
                    .clone()
                    .unwrap()
                    .contract_address,
                self.self_chain.clone(),
            ))
        } else {
            self.original_storage_mapping.get(&(
                cd.source_nft_contract_address.clone(),
                cd.source_chain.clone(),
            ))
        };
        let has_storage = storage.is_some();

        match (has_duplicate, has_storage) {
            (true, true) => {
                let dc = duplicate_collection_address.clone().unwrap();
                let dext =
                    external::ext_nft::ext(AccountId::from_str(&dc.contract_address).unwrap());
                dext.nft_tokens_for_owner(storage.clone().unwrap(), None, None)
                    .then(Self::ext(env::current_account_id()).query_owner_callback(
                        cd,
                        hexeh,
                        storage.unwrap(),
                        dc.contract_address.try_into().unwrap(),
                    ))
            }
            (true, false) => {
                let dc = duplicate_collection_address.clone().unwrap();
                let coll: AccountId = dc.contract_address.try_into().unwrap();
                let nft_collection = external::ext_nft::ext(coll.clone());
                let mut royalty = HashMap::new();
                royalty.insert(cd.royalty_receiver, cd.royalty.into());
                nft_collection
                    .nft_mint(
                        cd.token_id.clone(),
                        TokenMetadata {
                            media: Some(cd.metadata),
                            title: Some(cd.name),
                            description: None,
                            media_hash: None,
                            copies: None,
                            issued_at: None,
                            expires_at: None,
                            starts_at: None,
                            updated_at: None,
                            extra: None,
                            reference: None,
                            reference_hash: None,
                        },
                        cd.destination_user_address,
                        Some(royalty),
                    )
                    .then(Self::ext(env::current_account_id()).finalize_claim_callback(hexeh))
                    .then(Self::ext(env::current_account_id()).emit_claimed_event(
                        coll,
                        cd.token_id,
                        cd.transaction_hash,
                        cd.source_chain,
                    ))
            }
            (false, false) => {
                let nft_factory =
                    external::collection_factory::ext(self.collection_factory.clone());

                nft_factory
                    .deploy_nft_collection(cd.name.clone(), cd.symbol.clone())
                    .then(
                        Self::ext(env::current_account_id())
                            .after_collection_deploy_callback(cd, hexeh),
                    )
            }
            (false, true) => {
                let dext = external::ext_nft::ext(
                    cd.source_nft_contract_address.clone().try_into().unwrap(),
                );
                dext.nft_tokens_for_owner(storage.clone().unwrap(), None, None)
                    .then(Self::ext(env::current_account_id()).query_owner_callback(
                        cd.clone(),
                        hexeh,
                        storage.unwrap(),
                        cd.source_nft_contract_address.try_into().unwrap(),
                    ))
            }
        }
    }

    #[private]
    pub fn after_collection_deploy_callback(
        &mut self,
        cd: ClaimData,
        identifier: String,
        #[callback] result: Result<AccountId, PromiseError>,
    ) -> Promise {
        if result.is_err() {
            env::panic_str(&format!("Failed to deploy Collection: {:?}", result));
        }
        let collection = result.unwrap();
        self.original_to_duplicate_mapping.insert(
            &(
                cd.source_nft_contract_address.clone(),
                cd.source_chain.clone(),
            ),
            &ContractInfo {
                contract_address: collection.clone().to_string(),
                chain: self.self_chain.clone(),
            },
        );

        self.duplicate_to_original_mapping.insert(
            &(collection.clone(), self.self_chain.clone()),
            &ContractInfo {
                contract_address: cd.source_nft_contract_address.clone(),
                chain: cd.source_chain.clone(),
            },
        );
        let mut royalty = HashMap::new();
        royalty.insert(cd.royalty_receiver, cd.royalty.into());
        external::ext_nft::ext(collection.clone())
            .nft_mint(
                cd.token_id.clone(),
                TokenMetadata {
                    media: Some(cd.metadata),
                    title: Some(cd.name),
                    description: None,
                    media_hash: None,
                    copies: None,
                    issued_at: None,
                    expires_at: None,
                    starts_at: None,
                    updated_at: None,
                    extra: None,
                    reference: None,
                    reference_hash: None,
                },
                cd.destination_user_address,
                Some(royalty),
            )
            .then(Self::ext(env::current_account_id()).finalize_claim_callback(identifier))
            .then(Self::ext(env::current_account_id()).emit_claimed_event(
                collection,
                cd.token_id,
                cd.transaction_hash,
                cd.source_chain,
            ))
    }

    #[private]
    pub fn emit_claimed_event(
        &self,
        contract: AccountId,
        token_id: TokenId,
        transaction_hash: String,
        source_chain: String,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        require!(result.is_ok(), "Failed to emit claimed event");
        self.emit_event(EventLogVariant::Claimed(events::ClaimedEvent {
            contract,
            token_id,
            transaction_hash,
            source_chain,
        }));
    }

    #[private]
    pub fn finalize_claim_callback(
        &mut self,
        identifier: String,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        if result.is_ok() {
            self.unique_identifiers.insert(&identifier, &true);
            return;
        }
        env::panic_str(&format!("Failed to finalize claim. Reason: {:?}", result));
    }

    #[private]
    pub fn query_owner_callback(
        &mut self,
        cd: ClaimData,
        identifier: String,
        storage: AccountId,
        collection: AccountId,
        #[callback_result] result: Result<Vec<JsonToken>, PromiseError>,
    ) -> Promise {
        let is_stored = {
            let is_ok = result.is_ok();
            if is_ok {
                let tokens = result.unwrap();
                tokens.iter().find(|t| t.token_id == cd.token_id).is_some()
            } else {
                false
            }
        };
        if is_stored {
            external::nft_storage::ext(storage)
                .unlock_token(cd.destination_user_address, cd.token_id.clone())
                .then(Self::ext(env::current_account_id()).finalize_claim_callback(identifier))
                .then(Self::ext(env::current_account_id()).emit_claimed_event(
                    collection,
                    cd.token_id,
                    cd.transaction_hash,
                    cd.source_chain,
                ))
        } else {
            let mut royalty = HashMap::new();
            royalty.insert(cd.royalty_receiver, cd.royalty.into());
            external::ext_nft::ext(collection.clone())
                .nft_mint(
                    cd.token_id.clone(),
                    TokenMetadata {
                        media: Some(cd.metadata),
                        title: Some(cd.name),
                        description: None,
                        media_hash: None,
                        copies: None,
                        issued_at: None,
                        expires_at: None,
                        starts_at: None,
                        updated_at: None,
                        extra: None,
                        reference: None,
                        reference_hash: None,
                    },
                    cd.destination_user_address,
                    Some(royalty),
                )
                .then(Self::ext(env::current_account_id()).finalize_claim_callback(identifier))
                .then(Self::ext(env::current_account_id()).emit_claimed_event(
                    collection,
                    cd.token_id,
                    cd.transaction_hash,
                    cd.source_chain,
                ))
        }
    }

    fn reward_validators(&mut self, fee: u128, validators: Vec<Vec<u8>>) {
        let fee_per_head = fee / validators.len() as u128;
        require!(
            env::account_balance() >= NearToken::from_yoctonear(fee),
            "No rewards available"
        );
        for validator in validators {
            let k = String::from_utf8(validator).unwrap();
            // We know validator cannot be none.
            let val = self.validators.get(&k).unwrap();
            self.validators.insert(&k, &(val + fee_per_head));
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
