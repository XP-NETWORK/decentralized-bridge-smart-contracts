use std::{collections::HashMap, str::FromStr};

use events::{
    BridgeUpgraded, EventLog, EventLogVariant, NewValidatorAdded, ValidatorBlacklisted,
    ValidatorRewardsClaimed,
};
use external::nft_types::{TokenId, TokenMetadata};
use near_sdk::{
    collections::{LookupMap, TreeMap},
    env::{self, sha256},
    near, require, AccountId, NearToken, Promise, PromiseError,
};

pub mod types;
pub const COLLECTION: usize = include_bytes!("../../target/near/nft/nft.wasm").len();
pub const STORAGE: usize = include_bytes!("../../target/near/storage/storage.wasm").len();

use serde_json::to_string;
use types::{
    AddValidator, BlacklistValidator, ClaimData, ContractInfo, SignerAndSignature, Validator,
};
pub mod external;
#[near(contract_state)]
pub struct Bridge {
    collection_factory: AccountId,
    storage_factory: AccountId,
    validators: TreeMap<String, Validator>,
    blacklisted_validators: LookupMap<String, bool>,
    chain_id: String,
    duplicate_to_original_mapping: LookupMap<(AccountId, String), ContractInfo>,
    original_to_duplicate_mapping: LookupMap<(String, String), ContractInfo>,
    original_storage_mapping: LookupMap<(String, String), AccountId>,
    duplicate_storage_mapping: LookupMap<(String, String), AccountId>,
    unique_identifiers: LookupMap<String, bool>,
    unique_implementations: LookupMap<Vec<u8>, bool>
}

mod events;

impl Default for Bridge {
    fn default() -> Self {
        env::panic_str("Contract should be initialized before usage")
    }
}

pub const CHAIN_ID: &str = "NEAR";
pub const NFT_TYPE_SINGULAR: &str = "singular";

#[near]
impl Bridge {
    #[init]
    pub fn new(
        collection_factory: AccountId,
        storage_factory: AccountId,
        validators: Vec<(String, AccountId)>,
    ) -> Self {
        let mut v = TreeMap::new(b"v");
        for (pubk, acc_id) in validators {
            v.insert(
                &pubk,
                &Validator {
                    account_id: acc_id,
                    pending_rewards: 0,
                },
            );
        }
        Self {
            validators: v,
            collection_factory,
            storage_factory,
            blacklisted_validators: LookupMap::new(b"b"),
            chain_id: CHAIN_ID.to_string(),
            duplicate_to_original_mapping: LookupMap::new(b"d"),
            original_storage_mapping: LookupMap::new(b"o"),
            duplicate_storage_mapping: LookupMap::new(b"a"),
            unique_identifiers: LookupMap::new(b"u"),
            original_to_duplicate_mapping: LookupMap::new(b"r"),
            unique_implementations: LookupMap::new(b"i")
        }
    }

    pub fn validator_count(&self) -> u128 {
        self.validators.len() as u128
    }

    pub fn collection_factory(&self) -> AccountId {
        self.collection_factory.clone()
    }

    pub fn storage_factory(&self) -> AccountId {
        self.storage_factory.clone()
    }

    pub fn verify_signatures(
        &mut self,
        message: Vec<u8>,
        signer_and_signature: Vec<SignerAndSignature>,
    ) -> Vec<String> {
        let mut percentage = 0;
        let mut validators_to_reward = Vec::new();
        let mut unique = HashMap::new();
        for ss in signer_and_signature {
            if let None = unique.get(&ss.signer) {
                let valid = near_sdk::env::ed25519_verify(
                    &ss.signature.try_into().unwrap(),
                    &message,
                    &str_to_pubkey(&ss.signer),
                );
                if valid {
                    unique.insert(ss.signer.clone(), true);
                    percentage += 1;
                    validators_to_reward.push(ss.signer);
                }
            }
        }
        require!(percentage >= self.threshold(), "Insufficient signatures");
        validators_to_reward
    }

    pub fn add_validator(&mut self, validator: AddValidator, signatures: Vec<SignerAndSignature>) {
        require!(
            !self
                .blacklisted_validators
                .contains_key(&validator.public_key),
            "Validator is blacklisted"
        );
        require!(
            !self.validators.contains_key(&validator.public_key),
            "Validator already exists"
        );
        let serialized =
            near_sdk::borsh::to_vec(&validator).expect("Failed to serialize AddValidator into vec");
        self.verify_signatures(serialized, signatures);
        self.validators.insert(
            &validator.public_key,
            &Validator {
                account_id: validator.account_id,
                pending_rewards: 0,
            },
        );
        env::log_str(
            &to_string(&EventLog {
                standard: "bridge".to_string(),
                version: "1.0.0".to_string(),
                event: EventLogVariant::ValidatorAdded(NewValidatorAdded {
                    validator: validator.public_key,
                }),
            })
            .unwrap(),
        );
    }

    pub fn validator(&self, public_key: String) -> Option<Validator> {
        self.validators.get(&public_key)
    }

    pub fn claim_validator_rewards(&mut self, public_key: String) {
        let val = self.validators.get(&public_key).unwrap();
        require!(val.pending_rewards > 0, "No rewards to claim");
        require!(
            val.account_id == env::signer_account_id(),
            "Invalid withdrawal signer"
        );
        require!(
            env::account_balance() >= NearToken::from_yoctonear(val.pending_rewards),
            "No rewards available"
        );
        Promise::new(val.account_id)
            .transfer(NearToken::from_yoctonear(val.pending_rewards))
            .then(
                Self::ext(env::current_account_id()).claim_validator_rewards_callback(public_key),
            );
    }

    pub fn claim_validator_rewards_callback(
        &mut self,
        public_key: String,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        require!(result.is_ok(), "Failed to claim rewards");
        let v = self.validators.get(&public_key).unwrap();
        self.validators.insert(
            &public_key,
            &Validator {
                account_id: v.account_id.clone(),
                pending_rewards: 0,
            },
        );
        self.emit_event(EventLogVariant::ValidatorRewardsClaimed(
            ValidatorRewardsClaimed {
                amount: NearToken::from_yoctonear(v.pending_rewards),
                validator: v.account_id,
            },
        ));
    }

    #[payable]
    pub fn lock_nft(
        &mut self,
        source_nft_contract_address: AccountId,
        token_id: external::nft_types::TokenId,
        destination_chain: String,
        destination_address: String,
        metadata_uri: String,
    ) {
        require!(
            env::attached_deposit() >= NearToken::from_near(1),
            "Insufficient fee for deploying storage contract"
        );
        require!(
            destination_chain != self.chain_id,
            "Destination chain is the same as source chain"
        );
        let oca = self
            .duplicate_to_original_mapping
            .get(&(source_nft_contract_address.clone(), self.chain_id.clone()));

        match oca {
            Some(_) => {
                Self::check_storage_nft(
                    self.chain_id.clone(),
                    source_nft_contract_address,
                    token_id,
                    destination_chain,
                    destination_address,
                    &mut self.duplicate_storage_mapping,
                    self.storage_factory.clone(),
                    metadata_uri,
                    false,
                );
            }
            None => {
                Self::check_storage_nft(
                    self.chain_id.clone(),
                    source_nft_contract_address,
                    token_id,
                    destination_chain,
                    destination_address,
                    &mut self.original_storage_mapping,
                    self.storage_factory.clone(),
                    metadata_uri,
                    true,
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
        metadata_uri: String,
        original: bool,
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
                    metadata_uri,
                )),
            None => {
                let cost = env::storage_byte_cost()
                    .saturating_mul(STORAGE as u128)
                    .saturating_mul(5)
                    .saturating_div(4);
                external::storage_factory::ext(sf)
                    .with_attached_deposit(cost)
                    .deploy_nft_storage(source_nft_contract_address.clone())
                    .then(Self::ext(env::current_account_id()).transfer_to_storage(
                        source_nft_contract_address,
                        token_id,
                        destination_chain,
                        destination_address,
                        metadata_uri,
                        original,
                    ))
            }
        }
    }
    #[private]
    pub fn transfer_to_storage(
        &mut self,
        source_nft_contract_address: AccountId,
        token_id: TokenId,
        destination_chain: String,
        destination_user_address: String,
        metadata_uri: String,
        original: bool,
        #[callback_result] result: Result<AccountId, PromiseError>,
    ) -> Promise {
        let storage = if original {
            &mut self.original_storage_mapping
        } else {
            &mut self.duplicate_storage_mapping
        };
        match result {
            Ok(storage_address) => {
                storage.insert(
                    &(
                        source_nft_contract_address.to_string(),
                        self.chain_id.clone(),
                    ),
                    &storage_address,
                );
                external::ext_nft::ext(source_nft_contract_address.clone())
                    .with_attached_deposit(NearToken::from_yoctonear(1))
                    .nft_transfer(storage_address, token_id.clone(), None, None)
                    .then(Self::ext(env::current_account_id()).emit_locked_event(
                        destination_chain,
                        destination_user_address,
                        source_nft_contract_address,
                        token_id,
                        metadata_uri,
                    ))
            }
            Err(e) => env::panic_str(&format!("Failed to deploy Collection: {:?}", e)),
        }
    }

    #[payable]
    pub fn claim_nft(&mut self, cd: ClaimData, signatures: Vec<SignerAndSignature>) -> Promise {
        assert!(
            env::attached_deposit() >= NearToken::from_yoctonear(cd.fee.into()),
            "Insufficient fee"
        );
        assert!(
            cd.destination_chain == self.chain_id,
            "Invalid destination chain"
        );
        assert!(cd.nft_type == NFT_TYPE_SINGULAR, "Invalid NFT type");
        let serialized = near_sdk::borsh::to_vec(&cd).unwrap();
        let hash = sha256(&serialized);
        let hexeh = bytes2hex(&hash);
        require!(
            self.unique_identifiers.get(&hexeh).is_none(),
            "Data already processed!"
        );
        let validators_to_reward = self.verify_signatures(serialized, signatures);

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
                self.chain_id.clone(),
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
                        validators_to_reward,
                    ))
            }
            (true, false) => {
                let dc = duplicate_collection_address.clone().unwrap();
                let coll: AccountId = dc.contract_address.try_into().unwrap();
                let nft_collection = external::ext_nft::ext(coll.clone());
                let mut royalty = HashMap::new();
                royalty.insert(cd.royalty_receiver, cd.royalty.into());
                nft_collection
                    .with_attached_deposit(NearToken::from_yoctonear(10000000000000000000000))
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
                    .then(
                        Self::ext(env::current_account_id()).finalize_claim_callback(
                            hexeh,
                            cd.fee.into(),
                            validators_to_reward,
                        ),
                    )
                    .then(Self::ext(env::current_account_id()).emit_claimed_event(
                        coll,
                        cd.token_id,
                        cd.transaction_hash,
                        cd.source_chain,
                        cd.lock_tx_chain,
                    ))
            }
            (false, false) => {
                let nft_factory =
                    external::collection_factory::ext(self.collection_factory.clone());
                let cost = env::storage_byte_cost()
                    .saturating_mul(COLLECTION as u128)
                    .saturating_mul(5)
                    .saturating_div(4);
                nft_factory
                    .with_attached_deposit(cost)
                    .deploy_nft_collection(cd.name.clone(), cd.symbol.clone())
                    .then(
                        Self::ext(env::current_account_id()).after_collection_deploy_callback(
                            cd,
                            hexeh,
                            validators_to_reward,
                        ),
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
                        validators_to_reward,
                    ))
            }
        }
    }

    #[private]
    pub fn after_collection_deploy_callback(
        &mut self,
        cd: ClaimData,
        identifier: String,
        validators_to_reward: Vec<String>,
        #[callback_result] result: Result<AccountId, PromiseError>,
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
                chain: self.chain_id.clone(),
            },
        );

        self.duplicate_to_original_mapping.insert(
            &(collection.clone(), self.chain_id.clone()),
            &ContractInfo {
                contract_address: cd.source_nft_contract_address.clone(),
                chain: cd.source_chain.clone(),
            },
        );
        let mut royalty = HashMap::new();
        royalty.insert(cd.royalty_receiver, cd.royalty.into());
        external::ext_nft::ext(collection.clone())
            .with_attached_deposit(NearToken::from_yoctonear(10000000000000000000000))
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
            .then(
                Self::ext(env::current_account_id()).finalize_claim_callback(
                    identifier,
                    cd.fee.into(),
                    validators_to_reward,
                ),
            )
            .then(Self::ext(env::current_account_id()).emit_claimed_event(
                collection,
                cd.token_id,
                cd.transaction_hash,
                cd.source_chain,
                cd.lock_tx_chain,
            ))
    }

    #[private]
    pub fn emit_claimed_event(
        &mut self,
        collection: AccountId,
        token_id: TokenId,
        transaction_hash: String,
        source_chain: String,
        lock_tx_chain: String,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        require!(result.is_ok(), "Failed to emit claimed event");
        self.emit_event(EventLogVariant::Claimed(events::ClaimedEvent {
            contract: collection,
            token_id,
            transaction_hash,
            source_chain,
            lock_tx_chain,
        }));
    }

    #[private]
    pub fn finalize_claim_callback(
        &mut self,
        identifier: String,
        fee: u128,
        validators_to_reward: Vec<String>,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        if result.is_ok() {
            self.unique_identifiers.insert(&identifier, &true);
            self.reward_validators(fee, validators_to_reward);
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
        validators_to_reward: Vec<String>,
        #[callback_result] result: Result<Vec<external::nft_types::JsonToken>, PromiseError>,
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
                .then(
                    Self::ext(env::current_account_id()).finalize_claim_callback(
                        identifier,
                        cd.fee.into(),
                        validators_to_reward,
                    ),
                )
                .then(Self::ext(env::current_account_id()).emit_claimed_event(
                    collection,
                    cd.token_id,
                    cd.transaction_hash,
                    cd.source_chain,
                    cd.lock_tx_chain,
                ))
        } else {
            let mut royalty = HashMap::new();
            royalty.insert(cd.royalty_receiver, cd.royalty.into());
            external::ext_nft::ext(collection.clone())
                .with_attached_deposit(NearToken::from_yoctonear(10000000000000000000000))
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
                .then(
                    Self::ext(env::current_account_id()).finalize_claim_callback(
                        identifier,
                        cd.fee.into(),
                        validators_to_reward,
                    ),
                )
                .then(Self::ext(env::current_account_id()).emit_claimed_event(
                    collection,
                    cd.token_id,
                    cd.transaction_hash,
                    cd.source_chain,
                    cd.lock_tx_chain,
                ))
        }
    }

    fn reward_validators(&mut self, fee: u128, validators: Vec<String>) {
        let fee_per_head = fee / validators.len() as u128;
        require!(
            env::account_balance() >= NearToken::from_yoctonear(fee),
            "No rewards available"
        );
        for validator in validators {
            // We know validator cannot be none.
            let val = self.validators.get(&validator).unwrap();
            self.validators.insert(
                &validator,
                &Validator {
                    account_id: val.account_id,
                    pending_rewards: (val.pending_rewards + fee_per_head),
                },
            );
        }
    }

    #[private]
    pub fn emit_locked_event(
        &mut self,
        destination_chain: String,
        destination_address: String,
        source_nft_contract_address: AccountId,
        token_id: TokenId,
        metadata_uri: String,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        require!(result.is_ok(), "NFT transfer failed");
        self.emit_event(EventLogVariant::Locked(events::LockedEvent {
            destination_chain,
            destination_user_address: destination_address,
            source_nft_contract_address: source_nft_contract_address.to_string(),
            token_id,
            nft_type: "singular".to_string(),
            source_chain: self.chain_id.clone(),
            token_amount: 1,
            metadata_uri,
        }));
    }

    pub fn blacklist_validator(
        &mut self,
        validator: BlacklistValidator,
        signatures: Vec<SignerAndSignature>,
    ) {
        require!(
            !self
                .blacklisted_validators
                .contains_key(&validator.public_key),
            "Validator is already blacklisted"
        );
        let serialized = near_sdk::borsh::to_vec(&validator).unwrap();
        self.verify_signatures(serialized, signatures);
        self.validators.remove(&validator.public_key);
        self.blacklisted_validators
            .insert(&validator.public_key, &true);
        self.emit_event(EventLogVariant::ValidatorBlacklisted(
            ValidatorBlacklisted {
                validator: validator.public_key,
            },
        ));
    }

    pub fn threshold(&self) -> u128 {
        ((self.validators.len() * 2) / 3) as u128 + 1
    }

    pub fn upgrade_contract(
        &mut self,
        code: Vec<u8>,
        signatures: Vec<SignerAndSignature>,
    ) -> Promise {
        let hashed = sha256(&code);
        if self.unique_implementations.get(&hashed).is_some() {
            env::panic_str("Contract already upgraded");
        }
        let verified = self.verify_signatures(hashed.clone(), signatures);
        require!(
            verified.len() >= self.threshold() as usize,
            "Insufficient signatures for upgrade"
        );
        Promise::new(env::current_account_id())
            .deploy_contract(code)
            .then(Self::ext(env::current_account_id()).emit_upgrade_event(hashed))
    }

    pub fn emit_upgrade_event(&mut self, hashed: Vec<u8>, #[callback_result] result: Result<(), PromiseError>) {
        match result {
            Ok(_) => {
                self.emit_event(EventLogVariant::BridgeUpgraded(BridgeUpgraded {
                    timestamp: env::block_timestamp(),
                    block_height: env::block_height()
                }));
                self.unique_implementations.insert(&hashed, &true);
            }
            Err(e) => env::panic_str(&format!("Failed to upgrade contract: {:?}", e)),
        }
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

fn hex2bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex).expect("Failed to decode hex")
}

fn bytes2hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

fn str_to_pubkey<const LEN: usize>(s: &str) -> [u8; LEN] {
    hex2bytes(s).try_into().unwrap()
}
