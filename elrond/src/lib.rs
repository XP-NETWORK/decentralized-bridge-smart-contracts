#![no_std]

pub mod types;
multiversx_sc::imports!();
use types::{ClaimData, ContractInfo, SignatureInfo, TokenInfo, Validator};
pub const TX_FEES: u8 = 0;
pub const SELF_CHAIN: &[u8] = b"MULTIVERSX";
pub const TYPE_ERC721: &[u8] = b"singular"; // a more general term to accomodate non-evm chains
pub const TYPE_ERC1155: &[u8] = b"multiple"; // a more general term to accomodate non-evm chains
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

#[multiversx_sc::contract]
pub trait BridgeContract {
    #[view(tokens)]
    #[storage_mapper("tokens")]
    fn tokens(&self) -> BiDiMapper<TokenInfo<Self::Api>, TokenInfo<Self::Api>>;

    #[view(validators)]
    #[storage_mapper("validators")]
    fn validators(&self, address: &ManagedAddress) -> VecMapper<Validator<Self::Api>>;

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
        #[indexed] source_chain: ManagedBuffer,
        #[indexed] transaction_hash: ManagedBuffer,
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
    fn require_fees(&self) -> SCResult<()> {
        require!(
            self.call_value().egld_value().gt(&0),
            "Tx Fees is required!"
        );
        Ok(())
    }

    #[inline]
    fn only_validator(&self) -> SCResult<()> {
        require!(false, "Failed to verify signature!");
        Ok(())
    }

    #[inline]
    fn matches_current_chain(&self, destination_chain: &ManagedBuffer) -> SCResult<()> {
        require!(
            destination_chain.eq(SELF_CHAIN),
            "Invalid destination chain"
        );
        Ok(())
    }

    #[inline]
    fn has_correct_fee(&self, fee: BigUint) -> SCResult<()> {
        let sent_fee = self.call_value().egld_value().clone_value();
        require!(fee.le(&sent_fee), "Fee and sent amount do not match");
        Ok(())
    }

    fn verify_ed25519(&self, key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
        let public_key = VerifyingKey::from_bytes(key);
        let signatures = Signature::from(signature);
        // Convert bytes to a PublicKey. This can fail if the byte slice has an incorrect length.
        // let public_key1 = match VerifyingKey::from_bytes(key) {
        //     Ok(pk) => pk,
        //     Err(_) => return false, // If the public key is invalid, return false.
        // };

        // // Convert bytes to a Signature. This can fail if the byte slice has an incorrect length.
        // let signature = match Signature::from_bytes(signature) {
        //     Ok(sig) => sig,
        //     Err(_) => return false, // If the signature is invalid, return false.
        // };

        // // Verify the signature. This returns `true` if it's valid, and `false` if not.

        let rese = public_key.unwrap().verify(message, &signatures).is_ok();
        rese
    }

    #[endpoint(addValidator)]
    fn add_validator(
        &self,
        new_validator_public_key: ManagedAddress,
        signatures: ManagedVec<SignatureInfo<Self::Api>>,
    ) -> SCResult<()> {
        // require!(args.new_validator_address., "Address cannot be zero address!");
        require!(signatures.len() > 0, "Must have signatures!");

        let mut percentage: u64 = 0;

        for arg in signatures.into_iter() {
            let mut dest_slice = [0u8; 64];
            let _ = arg.sig.load_slice(0, &mut dest_slice);

            let res = self.verify_ed25519(
                &arg.public_key.to_byte_array(),
                &new_validator_public_key.to_byte_array(),
                &dest_slice,
            );
            if res {
                let lene = self.validators(&arg.public_key).is_empty();
                if lene == false && self.validators(&arg.public_key).get(1).added {
                    percentage = percentage + 1;
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
        Ok(())
    }

    #[endpoint(claimValidatorRewards)]
    fn claim_validator_rewards(
        &self,
        validator: ManagedAddress,
        signatures: ManagedVec<SignatureInfo<Self::Api>>,
    ) -> SCResult<()> {
        // require!(args.new_validator_address., "Address cannot be zero address!");
        require!(signatures.len() > 0, "Must have signatures!");

        if self.validators(&validator).is_empty() == true {
            require!(false, "Validator does not exist!");
        };

        let mut percentage: u64 = 0;

        for arg in signatures.into_iter() {
            let mut dest_slice = [0u8; 64];
            let _ = arg.sig.load_slice(0, &mut dest_slice);

            let res = self.verify_ed25519(
                &arg.public_key.to_byte_array(),
                &validator.to_byte_array(),
                &dest_slice,
            );
            if res {
                let lene = self.validators(&arg.public_key).is_empty();
                if lene == false && self.validators(&arg.public_key).get(1).added {
                    percentage = percentage + 1;
                }
            }
        }

        let validators_count = self.validators_count().get();

        require!(
            percentage >= (((validators_count * 2) / 3) + 1),
            "Threshold not reached!"
        );

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
        Ok(())
    }

    #[payable("*")]
    #[endpoint(lock721)]
    fn lock721(
        &self,
        token_id: TokenIdentifier,
        destination_chain: ManagedBuffer,
        destination_user_address: ManagedBuffer,
        source_nft_contract_address: TokenIdentifier,
        nonce: u64,
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
            .then(|| self.tokens().get_id(&TokenInfo {
                token_id: nonce,
                chain: self_chain_buffer.clone(),
                contract_address: source_nft_contract_address.as_managed_buffer().clone(),
            }));

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
                );
            }
        }
    }

    #[payable("*")]
    #[endpoint(lock1155)]
    fn lock1155(
        &self,
        token_id: TokenIdentifier,
        destination_chain: ManagedBuffer,
        destination_user_address: ManagedBuffer,
        source_nft_contract_address: TokenIdentifier,
        amount: BigUint,
        nonce: u64,
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
            .then(|| self.tokens().get_id(&TokenInfo {
                token_id: nonce,
                chain: self_chain_buffer.clone(),
                contract_address: source_nft_contract_address.as_managed_buffer().clone(),
            }));

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
                );
            }
        }
    }

    #[payable("EGLD")]
    #[endpoint(claimNft721)]
    fn claim_nft721(
        &self,
        data: ClaimData<Self::Api>,
        signatures: ManagedVec<SignatureInfo<Self::Api>>,
        uris: MultiValue2<ManagedBuffer, ManagedBuffer>,
    ) -> SCResult<()> {
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

        let payment_amount = self.call_value().egld_value();

        let identifier = data.name.clone().concat(data.symbol.clone());

        let duplicate_collection_address_option = self.original_to_duplicate_mapping().get(&(
            data.source_nft_contract_address.clone(),
            data.source_chain.clone(),
        ));

        let mut mvuri = ManagedVec::new();
        let (img, metadata) = uris.into_tuple();
        mvuri.push(img);
        mvuri.push(metadata);

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
                            data.source_nft_contract_address.into(),
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

                        self.send().transfer_esdt_via_async_call(
                            data.destination_user_address,
                            TokenIdentifier::from(v.address),
                            nonce,
                            BigUint::from(1u64),
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
                            data.source_nft_contract_address.into(),
                        );
                    }
                    None => {
                        self.create_collection(
                            payment_amount.clone_value(),
                            identifier,
                            self.blockchain().get_sc_address(),
                            data.clone(),
                            mvuri,
                        );
                    }
                }
            }
        }

        self.claimed(data.source_chain, data.transaction_hash);

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint(claimNft1155)]
    fn claim_nft1155(
        &self,
        data: ClaimData<Self::Api>,
        signatures: ManagedVec<SignatureInfo<Self::Api>>,
        uris: MultiValue2<ManagedBuffer, ManagedBuffer>,
    ) -> SCResult<()> {
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

        let payment_amount = self.call_value().egld_value();

        let identifier = data.name.clone().concat(data.symbol.clone());

        let duplicate_collection_address_option = self.original_to_duplicate_mapping().get(&(
            data.source_nft_contract_address.clone(),
            data.source_chain.clone(),
        ));

        let mut mvuri = ManagedVec::new();
        let (img, metadata) = uris.into_tuple();
        mvuri.push(img);
        mvuri.push(metadata);

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
                                data.source_nft_contract_address.into(),
                                data.token_amount,
                            );
                        } else {
                            let to_mint = data.token_amount - balance_of_tokens.clone();
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

                            self.send().transfer_esdt_via_async_call(
                                data.destination_user_address.clone(),
                                TokenIdentifier::from(v.address),
                                nonce,
                                to_mint,
                            );
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

                        self.send().transfer_esdt_via_async_call(
                            data.destination_user_address,
                            TokenIdentifier::from(v.address),
                            nonce,
                            data.token_amount,
                        );
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
                                data.source_nft_contract_address.into(),
                                data.token_amount,
                            );
                        } else {
                            let to_mint = data.token_amount - balance_of_tokens.clone();
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

                            self.send().transfer_esdt_via_async_call(
                                data.destination_user_address.clone(),
                                TokenIdentifier::from(data.source_nft_contract_address),
                                nonce,
                                to_mint,
                            );
                        }
                    }
                    None => {
                        self.create_collection(
                            payment_amount.clone_value(),
                            identifier,
                            self.blockchain().get_sc_address(),
                            data.clone(),
                            mvuri,
                        );
                    }
                }
            }
        }

        self.claimed(data.source_chain, data.transaction_hash);

        Ok(())
    }

    fn unlock721(
        &self,
        to: ManagedAddress,
        nonce: u64,
        contract_address: TokenIdentifier,
    ) -> SCResult<()> {
        self.unlock721_event(to.clone(), nonce, contract_address.clone());
        self.send()
            .transfer_esdt_via_async_call(to, contract_address, nonce, BigUint::from(1u64));
    }

    fn unlock1155(
        &self,
        to: ManagedAddress,
        nonce: u64,
        contract_address: TokenIdentifier,
        amount: BigUint,
    ) -> SCResult<()> {
        self.unlock1155_event(to.clone(), nonce, contract_address.clone(), amount.clone());
        self.send()
            .transfer_esdt_via_async_call(to, contract_address, nonce, amount);
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
        owner: ManagedAddress,
        data: ClaimData<Self::Api>,
        mvuri: ManagedVec<ManagedBuffer>,
    ) {
        require!(
            self.collections(identifier.clone()).is_empty(),
            "Collection already exists"
        );

        // let payment_amount = self.call_value().egld_value();
        if data.nft_type.eq(TYPE_ERC721) {
            self.send()
                .esdt_system_sc_proxy()
                .issue_non_fungible(
                    payment,
                    &data.name,
                    &data.symbol,
                    NonFungibleTokenProperties {
                        can_change_owner: true,
                        can_freeze: true,
                        can_pause: true,
                        can_transfer_create_role: true,
                        can_upgrade: true,
                        can_wipe: true,
                        can_add_special_roles: true,
                    },
                )
                .async_call()
                .with_callback(
                    self.callbacks()
                        .esdt_set_special_roles(identifier, owner, data, mvuri),
                )
                .call_and_exit();
        } else if data.nft_type.eq(TYPE_ERC1155) {
            self.send()
                .esdt_system_sc_proxy()
                .issue_semi_fungible(
                    payment,
                    &data.name,
                    &data.symbol,
                    SemiFungibleTokenProperties {
                        can_change_owner: true,
                        can_freeze: true,
                        can_pause: true,
                        can_transfer_create_role: true,
                        can_upgrade: true,
                        can_wipe: true,
                        can_add_special_roles: true,
                    },
                )
                .async_call()
                .with_callback(
                    self.callbacks()
                        .esdt_set_special_roles(identifier, owner, data, mvuri),
                )
                .call_and_exit();
        }
    }

    #[callback]
    fn esdt_set_special_roles(
        &self,
        identifier: ManagedBuffer,
        owner: ManagedAddress,
        data: ClaimData<Self::Api>,
        mvuri: ManagedVec<ManagedBuffer>,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(tid) => {
                self.collections(identifier).set(tid.clone());
                self.send()
                    .esdt_system_sc_proxy()
                    .set_special_roles(
                        &owner,
                        &tid,
                        [EsdtLocalRole::NftBurn, EsdtLocalRole::NftCreate]
                            .iter()
                            .map(|e| e.clone()),
                    )
                    .async_call()
                    .with_callback(
                        self.callbacks()
                            .esdt_transfer_callback(owner, tid, data, mvuri),
                    )
                    .call_and_exit()
            }
            ManagedAsyncCallResult::Err(err) => {
                panic!(
                    "Error while issuing ESDT({}): {:?}",
                    err.err_code, err.err_msg
                );
            }
        }
    }

    #[callback]
    fn esdt_transfer_callback(
        &self,
        owner: ManagedAddress,
        tid: TokenIdentifier,
        data: ClaimData<Self::Api>,
        mvuri: ManagedVec<ManagedBuffer>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.send()
                    .esdt_system_sc_proxy()
                    .transfer_ownership(&tid, &owner)
                    .async_call()
                    .with_callback(self.callbacks().after_transfer_callback(tid, data, mvuri))
                    .call_and_exit();
            }
            ManagedAsyncCallResult::Err(err) => {
                panic!(
                    "Error setting special roles ESDT({}): {:?}",
                    err.err_code, err.err_msg
                );
            }
        };
    }

    #[callback]
    fn after_transfer_callback(
        &self,
        tid: TokenIdentifier,
        data: ClaimData<Self::Api>,
        mvuri: ManagedVec<ManagedBuffer>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) -> TokenIdentifier {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
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

                self.send().transfer_esdt_via_async_call(
                    data.destination_user_address,
                    tid,
                    nonce,
                    data.token_amount,
                );
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
