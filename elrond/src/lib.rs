#![no_std]

pub mod types;
multiversx_sc::imports!();
use types::{ClaimData, ContractInfo, SignatureInfo, TokenInfo, Validator};
pub const TX_FEES: u8 = 0;
pub const SELF_CHAIN: &[u8] = b"MULTIVERSX";
pub const TYPE_ERC721: &[u8] = b"singular"; // a more general term to accomodate non-evm chains
pub const TYPE_ERC1155: &[u8] = b"multiple"; // a more general term to accomodate non-evm chains
pub const COLLECTION_FEE: u64 = 50000000000000000u64;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

#[multiversx_sc::contract]
pub trait BridgeContract {
    #[view(tokens)]
    #[storage_mapper("tokens")]
    fn tokens(&self) -> BiDiMapper<TokenInfo<Self::Api>, TokenInfo<Self::Api>>;

    #[view(validators)]
    #[storage_mapper("validators")]
    fn validators(&self, address: &ManagedAddress) -> VecMapper<Validator<Self::Api>>;

    #[view(blacklistedValidators)]
    #[storage_mapper("blacklistedValidators")]
    fn blacklisted_validators(&self, address: &ManagedAddress) -> SingleValueMapper<bool>;

    #[view(validatorsCount)]
    #[storage_mapper("validatorsCount")]
    fn validators_count(&self) -> SingleValueMapper<u64>;

    #[view(uniqueIdentifier)]
    #[storage_mapper("uniqueIdentifier")]
    fn unique_identifier(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(originalToDuplicateMapping)]
    #[storage_mapper("originalToDuplicateMapping")]
    fn original_to_duplicate_mapping(
        &self,
    ) -> MapMapper<(ManagedBuffer, ManagedBuffer), ContractInfo<Self::Api>>;

    #[view(duplicateToOriginalMapping)]
    #[storage_mapper("duplicateToOriginalMapping")]
    fn duplicate_to_original_mapping(
        &self,
    ) -> MapMapper<(TokenIdentifier, ManagedBuffer), ContractInfo<Self::Api>>;

    #[event("AddNewValidator")]
    fn add_new_validator(&self, #[indexed] validator: ManagedAddress);

    #[event("Locked")]
    fn locked(
        &self,
        #[indexed] token_id: u64,
        #[indexed] destination_chain: ManagedBuffer,
        #[indexed] destination_user_address: ManagedBuffer,
        #[indexed] source_nft_contract_address: TokenIdentifier,
        #[indexed] token_amount: BigUint,
        #[indexed] nft_type: ManagedBuffer,
        #[indexed] chain: ManagedBuffer,
        #[indexed] metadata_uri: ManagedBuffer,
    );

    #[event("UnLock721")]
    fn unlock721_event(
        &self,
        #[indexed] to: ManagedAddress,
        #[indexed] token_id: u64,
        #[indexed] contract_address: TokenIdentifier,
    );

    #[event("UnLock1155")]
    fn unlock1155_event(
        &self,
        #[indexed] to: ManagedAddress,
        #[indexed] token_id: u64,
        #[indexed] contract_address: TokenIdentifier,
        #[indexed] amount: BigUint,
    );

    #[event("Claimed")]
    fn claimed(
        &self,
        #[indexed] lock_tx_chain: ManagedBuffer,
        #[indexed] source_chain: ManagedBuffer,
        #[indexed] transaction_hash: ManagedBuffer,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] nonce: u64,
        #[indexed] amt: BigUint,
        #[indexed] nft_type: ManagedBuffer,
    );

    #[event("RewardValidator")]
    fn reward_validator_event(&self, #[indexed] validator: ManagedAddress);

    #[init]
    fn init(&self, public_key: ManagedAddress) {
        self.validators(&public_key).push(&Validator {
            added: true,
            pending_reward: BigUint::from(0u64),
        });
        self.validators_count().update(|val| {
            *val += 1u64;
            *val
        });
    }

    #[inline]
    fn matches_current_chain(&self, destination_chain: &ManagedBuffer) {
        require!(
            destination_chain.eq(SELF_CHAIN),
            "Invalid destination chain"
        );
    }

    #[inline]
    fn has_correct_fee(&self, fee: BigUint) {
        let sent_fee = self.call_value().egld_value().clone_value();
        require!(sent_fee.ge(&fee), "Sent amount less than fee");
    }

    fn verify_ed25519(&self, key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
        let public_key = VerifyingKey::from_bytes(key);
        let signatures = Signature::from(signature);
        public_key.unwrap().verify(message, &signatures).is_ok()
    }

    #[endpoint(addValidator)]
    fn add_validator(
        &self,
        new_validator_public_key: ManagedAddress,
        signatures: ManagedVec<SignatureInfo<Self::Api>>,
    ) {
        require!(!self.blacklisted_validators(&new_validator_public_key).get(), "validator blacklisted");

        require!(signatures.len() > 0, "Must have signatures!");
        let mut uv = ManagedMap::new();

        let mut percentage: u64 = 0;

        for arg in signatures.into_iter() {
            let mut dest_slice = [0u8; 64];
            let _ = arg.sig.load_slice(0, &mut dest_slice);

            let valid = self.verify_ed25519(
                &arg.public_key.to_byte_array(),
                &new_validator_public_key.to_byte_array(),
                &dest_slice,
            );
            if valid {
                let validator_exists = self.validators(&arg.public_key).is_empty();
                if !validator_exists && !uv.contains(&arg.public_key.as_managed_buffer()) {
                    percentage = percentage + 1;
                    uv.put(
                        &arg.public_key.as_managed_buffer(),
                        &ManagedBuffer::new_from_bytes(b"true"),
                    );
                }
            }
        }

        let validators_count = self.validators_count().get();

        require!(
            percentage >= (((validators_count * 2) / 3) + 1),
            "Threshold not reached!"
        );

        self.add_new_validator(new_validator_public_key.clone());
        self.validators(&new_validator_public_key).push(&Validator {
            added: true,
            pending_reward: BigUint::from(0u64),
        });
        self.validators_count().update(|val| {
            *val += 1u64;
            *val
        });
    }

    #[endpoint(blacklistValidator)]
    fn blacklist_validator(
        &self,
        address: ManagedAddress,
        signatures: ManagedVec<SignatureInfo<Self::Api>>,
    ) {
        require!(signatures.len() > 0, "Must have signatures!");
        let mut uv = ManagedMap::new();
        let mut percentage: u64 = 0;

        for arg in signatures.into_iter() {
            let mut dest_slice = [0u8; 64];
            let _ = arg.sig.load_slice(0, &mut dest_slice);

            let valid = self.verify_ed25519(
                &arg.public_key.to_byte_array(),
                &address.to_byte_array(),
                &dest_slice,
            );
            if valid {
                let validator_exists = self.validators(&arg.public_key).is_empty();
                if !validator_exists && !uv.contains(&arg.public_key.as_managed_buffer()) {
                    percentage = percentage + 1;
                    uv.put(
                        &arg.public_key.as_managed_buffer(),
                        &ManagedBuffer::new_from_bytes(b"true"),
                    );
                }
            }
        }

        let validators_count = self.validators_count().get();

        require!(
            percentage >= (((validators_count * 2) / 3) + 1),
            "Threshold not reached!"
        );

        self.validators(&address).clear();
        self.validators_count().update(|val| {
            *val -= 1u64;
            *val
        });
        self.blacklisted_validators(&address).set(true);
    }

    #[endpoint(claimValidatorRewards)]
    fn claim_validator_rewards(&self, validator: ManagedAddress) {
        if self.validators(&validator).is_empty() == true {
            require!(false, "Validator does not exist!");
        };
        self.reward_validator_event(validator.clone());
        let rewards = self.validators(&validator).get(1).pending_reward;

        self.send().direct_egld(&validator, &rewards);
        self.validators(&validator).set(
            1,
            &Validator {
                added: true,
                pending_reward: BigUint::from(0u64),
            },
        );
    }

    #[payable("*")]
    #[endpoint(lock721)]
    fn lock721(
        &self,
        _token_id: TokenIdentifier,
        destination_chain: ManagedBuffer,
        destination_user_address: ManagedBuffer,
        source_nft_contract_address: TokenIdentifier,
        nonce: u64,
        metadata_uri: ManagedBuffer,
    ) {
        let self_chain = SELF_CHAIN;
        let self_chain_buffer: ManagedBuffer = self_chain.into();
        let nft_type = ManagedBuffer::new_from_bytes(TYPE_ERC721);

        // send duplicate and get original
        let original_collection_address_option = self.duplicate_to_original_mapping().get(&(
            source_nft_contract_address.clone(),
            self_chain_buffer.clone(),
        ));

        let data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &source_nft_contract_address,
            nonce,
        );
        require!(data.amount.eq(&BigUint::from(1u64)), "No nft received");

        let is_token_exists = self
            .tokens()
            .contains_value(&TokenInfo {
                token_id: nonce,
                chain: self_chain_buffer.clone(),
                contract_address: source_nft_contract_address.as_managed_buffer().clone(),
            })
            .then(|| {
                self.tokens().get_id(&TokenInfo {
                    token_id: nonce,
                    chain: self_chain_buffer.clone(),
                    contract_address: source_nft_contract_address.as_managed_buffer().clone(),
                })
            });

        let mut mut_token_id: u64 = 0;

        match is_token_exists {
            Some(v) => {
                mut_token_id = v.token_id;
            }
            None => {
                mut_token_id = nonce;
                self.tokens().insert(
                    TokenInfo {
                        token_id: nonce,
                        chain: self_chain_buffer.clone(),
                        contract_address: source_nft_contract_address.as_managed_buffer().clone(),
                    },
                    TokenInfo {
                        token_id: nonce,
                        chain: self_chain_buffer.clone(),
                        contract_address: source_nft_contract_address.as_managed_buffer().clone(),
                    },
                );
            }
        }

        match original_collection_address_option {
            Some(v) => {
                // notOriginal
                self.locked(
                    mut_token_id,
                    destination_chain,
                    destination_user_address,
                    TokenIdentifier::from(v.address),
                    BigUint::from(1u64),
                    nft_type,
                    v.chain,
                    metadata_uri,
                );
            }
            None => {
                // isOriginal
                self.locked(
                    mut_token_id,
                    destination_chain,
                    destination_user_address,
                    source_nft_contract_address,
                    BigUint::from(1u64),
                    nft_type,
                    SELF_CHAIN.into(),
                    metadata_uri,
                );
            }
        }
    }

    #[payable("*")]
    #[endpoint(lock1155)]
    fn lock1155(
        &self,
        _token_id: TokenIdentifier,
        destination_chain: ManagedBuffer,
        destination_user_address: ManagedBuffer,
        source_nft_contract_address: TokenIdentifier,
        amount: BigUint,
        nonce: u64,
        metadata_uri: ManagedBuffer,
    ) {
        let self_chain = SELF_CHAIN;
        let self_chain_buffer: ManagedBuffer = self_chain.into();
        let nft_type = ManagedBuffer::new_from_bytes(TYPE_ERC1155);

        // send duplicate and get original
        let original_collection_address_option = self.duplicate_to_original_mapping().get(&(
            source_nft_contract_address.clone(),
            self_chain_buffer.clone(),
        ));

        let data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &source_nft_contract_address,
            nonce,
        );
        require!(data.amount.eq(&amount), "No sft received");

        let is_token_exists = self
            .tokens()
            .contains_value(&TokenInfo {
                token_id: nonce,
                chain: self_chain_buffer.clone(),
                contract_address: source_nft_contract_address.as_managed_buffer().clone(),
            })
            .then(|| {
                self.tokens().get_id(&TokenInfo {
                    token_id: nonce,
                    chain: self_chain_buffer.clone(),
                    contract_address: source_nft_contract_address.as_managed_buffer().clone(),
                })
            });

        let mut mut_token_id: u64 = 0;

        match is_token_exists {
            Some(v) => {
                mut_token_id = v.token_id;
            }
            None => {
                mut_token_id = nonce;
                self.tokens().insert(
                    TokenInfo {
                        token_id: nonce,
                        chain: self_chain_buffer.clone(),
                        contract_address: source_nft_contract_address.as_managed_buffer().clone(),
                    },
                    TokenInfo {
                        token_id: nonce,
                        chain: self_chain_buffer.clone(),
                        contract_address: source_nft_contract_address.as_managed_buffer().clone(),
                    },
                );
            }
        }

        match original_collection_address_option {
            Some(v) => {
                // notOriginal
                self.locked(
                    mut_token_id,
                    destination_chain,
                    destination_user_address,
                    TokenIdentifier::from(v.address),
                    amount,
                    nft_type,
                    v.chain,
                    metadata_uri,
                );
            }
            None => {
                // isOriginal
                self.locked(
                    mut_token_id,
                    destination_chain,
                    destination_user_address,
                    source_nft_contract_address,
                    amount,
                    nft_type,
                    SELF_CHAIN.into(),
                    metadata_uri,
                );
            }
        }
    }

    #[payable("EGLD")]
    #[endpoint(claimNft721)]
    fn claim_nft721(
        &self,
        data: ClaimData<Self::Api>,
        signatures: ManagedVec<SignatureInfo<Self::Api>>
    ) {
        let _ = self.has_correct_fee(data.fee.clone());
        let _ = self.matches_current_chain(&data.destination_chain);

        require!(data.nft_type.eq(TYPE_ERC721), "Invalid NFT type!");

        let hash = self.create_claim_data_hash(data.clone());
        require!(
            !self.unique_identifier().contains(&hash.clone()),
            "Data already processed!"
        );

        self.unique_identifier().insert(hash.clone());

        let validators_to_reward = self.verify_signatures(hash.clone(), signatures);

        self.reward_validators(data.fee.clone(), validators_to_reward);

        let sub = self.call_value().egld_value().to_u64().unwrap() - data.fee.to_u64().unwrap();

        let payment_amount = BigUint::from(sub);

        let identifier = data.name.clone().concat(data.symbol.clone());

        let duplicate_collection_address_option = self.original_to_duplicate_mapping().get(&(
            data.source_nft_contract_address.clone(),
            data.source_chain.clone(),
        ));

        let mut mvuri = ManagedVec::new();
        mvuri.push(data.img_uri.clone());
        mvuri.push(data.attrs.clone());

        let is_token_exists = self
            .tokens()
            .contains_id(&TokenInfo {
                token_id: data.token_id.parse_as_u64().unwrap().clone(),
                chain: data.source_chain.clone(),
                contract_address: data.source_nft_contract_address.clone(),
            })
            .then(|| {
                self.tokens().get_value(&TokenInfo {
                    token_id: data.token_id.parse_as_u64().unwrap().clone(),
                    chain: data.source_chain.clone(),
                    contract_address: data.source_nft_contract_address.clone(),
                })
            });

        match duplicate_collection_address_option {
            Some(v) => {
                // isDuplicate
                match is_token_exists {
                    Some(v) => {
                        let _ = self.unlock721(
                            data.destination_user_address,
                            v.token_id,
                            data.source_nft_contract_address.clone().into(),
                        );

                        self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                        self.claimed(
                            data.lock_tx_chain,
                            data.source_chain,
                            data.transaction_hash,
                            data.source_nft_contract_address.into(),
                            v.token_id,
                            BigUint::from(1u64),
                            data.nft_type
                        );
                    }
                    None => {
                        let nonce = self.send().esdt_nft_create(
                            &TokenIdentifier::from(v.address.clone()),
                            &(BigUint::from(1u64)),
                            &data.name,
                            &data.royalty,
                            &ManagedBuffer::new(),
                            &data.attrs,
                            &mvuri,
                        );

                        self.send().direct_esdt(
                            &data.destination_user_address,
                            &TokenIdentifier::from(v.address.clone()),
                            nonce,
                            &BigUint::from(1u64),
                        );

                        self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                        self.claimed(
                            data.lock_tx_chain,
                            data.source_chain,
                            data.transaction_hash,
                            TokenIdentifier::from(v.address.clone()),
                            nonce,
                            BigUint::from(1u64),
                            data.nft_type
                        );
                    }
                }
            }
            None => {
                // notDuplicate
                match is_token_exists {
                    Some(v) => {
                        let _ = self.unlock721(
                            data.destination_user_address,
                            v.token_id,
                            data.source_nft_contract_address.clone().into(),
                        );

                        self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                        self.claimed(
                            data.lock_tx_chain,
                            data.source_chain,
                            data.transaction_hash,
                            data.source_nft_contract_address.into(),
                            v.token_id,
                            BigUint::from(1u64),
                            data.nft_type
                        );
                    }
                    None => {
                        self.create_collection(
                            payment_amount,
                            identifier,
                            data.clone(),
                            mvuri,
                        );
                    }
                }
            }
        }
    }

    #[payable("EGLD")]
    #[endpoint(claimNft1155)]
    fn claim_nft1155(
        &self,
        data: ClaimData<Self::Api>,
        signatures: ManagedVec<SignatureInfo<Self::Api>>
    ) {
        let _ = self.has_correct_fee(data.fee.clone());
        let _ = self.matches_current_chain(&data.destination_chain);

        require!(data.nft_type.eq(TYPE_ERC1155), "Invalid NFT type!");

        let hash = self.create_claim_data_hash(data.clone());

        require!(
            !self.unique_identifier().contains(&hash.clone()),
            "Data already processed!"
        );

        self.unique_identifier().insert(hash.clone());

        let validators_to_reward = self.verify_signatures(hash.clone(), signatures);

        self.reward_validators(data.fee.clone(), validators_to_reward);

        let sub = self.call_value().egld_value().to_u64().unwrap() - data.fee.to_u64().unwrap();

        let payment_amount = BigUint::from(sub);

        let identifier = data.name.clone().concat(data.symbol.clone());

        let duplicate_collection_address_option = self.original_to_duplicate_mapping().get(&(
            data.source_nft_contract_address.clone(),
            data.source_chain.clone(),
        ));

        let mut mvuri = ManagedVec::new();
        mvuri.push(data.img_uri.clone());
        mvuri.push(data.attrs.clone());

        let is_token_exists = self
            .tokens()
            .contains_id(&TokenInfo {
                token_id: data.token_id.parse_as_u64().unwrap().clone(),
                chain: data.source_chain.clone(),
                contract_address: data.source_nft_contract_address.clone(),
            })
            .then(|| {
                self.tokens().get_value(&TokenInfo {
                    token_id: data.token_id.parse_as_u64().unwrap().clone(),
                    chain: data.source_chain.clone(),
                    contract_address: data.source_nft_contract_address.clone(),
                })
            });

        match duplicate_collection_address_option {
            Some(v) => {
                // isDuplicate
                match is_token_exists {
                    Some(vnonce) => {
                        let balance_of_tokens = self.blockchain().get_esdt_balance(
                            &self.blockchain().get_sc_address(),
                            &v.address.clone().into(),
                            data.token_id.parse_as_u64().unwrap(),
                        );

                        if balance_of_tokens >= data.token_amount {
                            let _ = self.unlock1155(
                                data.destination_user_address,
                                vnonce.token_id,
                                data.source_nft_contract_address.clone().into(),
                                data.token_amount.clone(),
                            );

                            self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                            self.claimed(
                                data.lock_tx_chain,
                                data.source_chain,
                                data.transaction_hash,
                                data.source_nft_contract_address.into(),
                                vnonce.token_id,
                                data.token_amount,
                                data.nft_type
                            )
                        } else {
                            let to_mint = data.token_amount.clone() - balance_of_tokens.clone();
                            let _ = self.unlock1155(
                                data.destination_user_address.clone(),
                                vnonce.token_id,
                                data.source_nft_contract_address.into(),
                                balance_of_tokens.into(),
                            );

                            let nonce = self.send().esdt_nft_create(
                                &TokenIdentifier::from(v.address.clone()),
                                &to_mint,
                                &data.name,
                                &data.royalty,
                                &ManagedBuffer::new(),
                                &data.attrs,
                                &mvuri,
                            );
                            self.send().direct_esdt(
                                &data.destination_user_address.clone(),
                                &&TokenIdentifier::from(v.address.clone()),
                                nonce,
                                &data.token_amount,
                            );

                            self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                            self.claimed(
                                data.lock_tx_chain,
                                data.source_chain,
                                data.transaction_hash,
                                TokenIdentifier::from(v.address.clone()),
                                nonce,
                                data.token_amount.clone(),
                                data.nft_type
                            )
                        }
                    }
                    None => {
                        let nonce = self.send().esdt_nft_create(
                            &TokenIdentifier::from(v.address.clone()),
                            &data.token_amount,
                            &data.name,
                            &data.royalty,
                            &ManagedBuffer::new(),
                            &data.attrs,
                            &mvuri,
                        );

                        self.send().direct_esdt(
                            &data.destination_user_address.clone(),
                            &TokenIdentifier::from(v.address.clone()),
                            nonce,
                            &data.token_amount,
                        );

                        self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                        self.claimed(
                            data.lock_tx_chain,
                            data.source_chain,
                            data.transaction_hash,
                            TokenIdentifier::from(v.address.clone()),
                            nonce,
                            data.token_amount.clone(),
                            data.nft_type
                        )
                    }
                }
            }
            None => {
                // notDuplicate
                match is_token_exists {
                    Some(vnonce) => {
                        // let _ = self.unlock1155(
                        //     data.destination_user_address,
                        //     v,
                        //     data.source_nft_contract_address.into(),
                        //     data.token_amount,
                        // );
                        let balance_of_tokens = self.blockchain().get_esdt_balance(
                            &self.blockchain().get_sc_address(),
                            &data.source_nft_contract_address.clone().into(),
                            data.token_id.parse_as_u64().unwrap(),
                        );

                        if balance_of_tokens >= data.token_amount {
                            let _ = self.unlock1155(
                                data.destination_user_address,
                                vnonce.token_id,
                                data.source_nft_contract_address.clone().into(),
                                data.token_amount.clone(),
                            );

                            self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                            self.claimed(
                                data.lock_tx_chain,
                                data.source_chain,
                                data.transaction_hash,
                                data.source_nft_contract_address.into(),
                                vnonce.token_id,
                                data.token_amount,
                                data.nft_type
                            )
                        } else {
                            let to_mint = data.token_amount.clone() - balance_of_tokens.clone();
                            let _ = self.unlock1155(
                                data.destination_user_address.clone(),
                                vnonce.token_id,
                                data.source_nft_contract_address.clone().into(),
                                balance_of_tokens.into(),
                            );

                            let nonce = self.send().esdt_nft_create(
                                &TokenIdentifier::from(data.source_nft_contract_address.clone()),
                                &to_mint,
                                &data.name,
                                &data.royalty,
                                &ManagedBuffer::new(),
                                &data.attrs,
                                &mvuri,
                            );

                            self.send().direct_esdt(
                                &data.destination_user_address.clone(),
                                &TokenIdentifier::from(data.source_nft_contract_address.clone()),
                                nonce,
                                &to_mint,
                            );

                            self.send().direct_egld(&self.blockchain().get_caller(), &BigUint::from(COLLECTION_FEE));

                            self.claimed(
                                data.lock_tx_chain,
                                data.source_chain,
                                data.transaction_hash,
                                data.source_nft_contract_address.into(),
                                nonce,
                                data.token_amount.clone(),
                                data.nft_type
                            )
                        }
                    }
                    None => {
                        self.create_collection(
                            payment_amount,
                            identifier,
                            data.clone(),
                            mvuri,
                        );
                    }
                }
            }
        }
    }

    fn unlock721(&self, to: ManagedAddress, nonce: u64, contract_address: TokenIdentifier) {
        self.unlock721_event(to.clone(), nonce, contract_address.clone());
        self.send()
            .direct_esdt(&to, &contract_address, nonce, &BigUint::from(1u64));
    }

    fn unlock1155(
        &self,
        to: ManagedAddress,
        nonce: u64,
        contract_address: TokenIdentifier,
        amount: BigUint,
    ) {
        self.unlock1155_event(to.clone(), nonce, contract_address.clone(), amount.clone());
        self.send()
            .direct_esdt(&to, &contract_address, nonce, &amount);
    }

    fn create_claim_data_hash(&self, data: ClaimData<Self::Api>) -> ManagedBuffer {
        let mut encoded_data = ManagedBuffer::new();
        data.dep_encode(&mut encoded_data).unwrap();

        let hash = self.crypto().keccak256(encoded_data);
        hash.as_managed_buffer().clone()
    }

    fn verify_signatures(
        &self,
        hash: ManagedBuffer,
        signatures: ManagedVec<SignatureInfo<Self::Api>>,
    ) -> ManagedVec<ManagedAddress> {
        let mut percentage: u64 = 0;
        let mut validators_to_reward: ManagedVec<ManagedAddress> = ManagedVec::new();

        for arg in signatures.into_iter() {
            let mut dest_slice = [0u8; 64];
            let _ = arg.sig.load_slice(0, &mut dest_slice);

            let mut dest_slice2 = [0u8; 32];
            let _ = hash.load_slice(0, &mut dest_slice2);

            let res =
                self.verify_ed25519(&arg.public_key.to_byte_array(), &dest_slice2, &dest_slice);
            if res {
                let lene = self.validators(&arg.public_key).is_empty();
                if lene == false && self.validators(&arg.public_key).get(1).added {
                    percentage = percentage + 1;
                    validators_to_reward.push(arg.public_key);
                }
            }
        }
        let validators_count = self.validators_count().get();

        require!(
            percentage >= (((validators_count * 2) / 3) + 1),
            "Threshold not reached!"
        );
        validators_to_reward
    }

    fn reward_validators(&self, fee: BigUint, validators_to_reward: ManagedVec<ManagedAddress>) {
        require!(fee.gt(&0), "Invalid fees");

        let total_rewards = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);

        require!(total_rewards >= fee, "No rewards available");

        let fee_per_validator = fee / BigUint::from(validators_to_reward.len());

        for address in validators_to_reward.into_iter() {
            let mut pendings = self.validators(&address).get(1).pending_reward;
            pendings += fee_per_validator.clone();
            self.validators(&address).set(
                1,
                &Validator {
                    added: true,
                    pending_reward: pendings,
                },
            )
        }
    }

    // COLLECTION

    #[view(collections)]
    #[storage_mapper("collections")]
    fn collections(&self, identifier: ManagedBuffer) -> SingleValueMapper<TokenIdentifier>;

    fn create_collection(
        &self,
        payment: BigUint,
        identifier: ManagedBuffer,
        data: ClaimData<Self::Api>,
        mvuri: ManagedVec<ManagedBuffer>,
    ) {
        require!(
            self.collections(identifier.clone()).is_empty(),
            "Collection already exists"
        );

        if data.nft_type.eq(TYPE_ERC721) {
            self.send()
                .esdt_system_sc_proxy()
                .issue_and_set_all_roles(
                    payment,
                    data.name.clone(),
                    data.symbol.clone(),
                    EsdtTokenType::NonFungible,
                    0,
                )
                .callback(self.callbacks().after_transfer_callback(data, mvuri))
                .async_call_and_exit();
        } else if data.nft_type.eq(TYPE_ERC1155) {
            self.send()
                .esdt_system_sc_proxy()
                .issue_and_set_all_roles(
                    payment,
                    data.name.clone(),
                    data.symbol.clone(),
                    EsdtTokenType::SemiFungible,
                    0,
                )
                .callback(self.callbacks().after_transfer_callback(data, mvuri))
                .async_call_and_exit();
        }
    }

    #[callback]
    fn after_transfer_callback(
        &self,
        data: ClaimData<Self::Api>,
        mvuri: ManagedVec<ManagedBuffer>,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(tid) => {
                let nonce = self.send().esdt_nft_create(
                    &tid,
                    &data.token_amount,
                    &data.name,
                    &data.royalty,
                    &ManagedBuffer::new(),
                    &data.attrs,
                    &mvuri,
                );

                self.original_to_duplicate_mapping().insert(
                    (
                        data.source_nft_contract_address.clone(),
                        data.source_chain.clone(),
                    ),
                    ContractInfo {
                        chain: ManagedBuffer::from(SELF_CHAIN),
                        address: tid.as_managed_buffer().clone(),
                    },
                );

                self.duplicate_to_original_mapping().insert(
                    (tid.clone(), ManagedBuffer::from(SELF_CHAIN)),
                    ContractInfo {
                        chain: data.source_chain.clone(),
                        address: data.source_nft_contract_address.clone(),
                    },
                );

                self.tokens().insert(
                    TokenInfo {
                        token_id: data.token_id.parse_as_u64().unwrap().clone(),
                        chain: data.source_chain.clone(),
                        contract_address: data.source_nft_contract_address.clone(),
                    },
                    TokenInfo {
                        token_id: nonce,
                        chain: ManagedBuffer::from(SELF_CHAIN),
                        contract_address: tid.as_managed_buffer().clone(),
                    },
                );

                self.send().direct_esdt(
                    &data.destination_user_address,
                    &tid,
                    nonce,
                    &BigUint::from(data.token_amount.clone()),
                );
                    self.claimed(
                        data.lock_tx_chain,
                        data.source_chain,
                        data.transaction_hash,
                        tid,
                        nonce,
                        data.token_amount,
                        data.nft_type
                    )
            }
            ManagedAsyncCallResult::Err(err) => {
                panic!(
                    "Error transferring of ESDT({}): {:?}",
                    err.err_code, err.err_msg
                );
            }
        }
    }
}
