use alloc::{string::String, vec::Vec};
use casper_types::{bytesrepr::{self, Bytes, FromBytes, ToBytes}, CLType, CLTyped, ContractHash, PublicKey, U256, U512};
use casper_types::account::AccountHash;
use casper_types::CLType::String;
use crate::external::xp_nft::TokenIdentifier;

pub struct Validator {
    pub added: bool,
    pub pending_rewards: U512,
}

impl FromBytes for Validator {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (added, remainder) = bool::from_bytes(bytes)?;
        let (pending_rewards, remainder) = U512::from_bytes(remainder)?;

        Ok((Self { added, pending_rewards }, remainder))
    }
}

impl ToBytes for Validator {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.added.to_bytes()?);
        result.extend(self.pending_rewards.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.added.serialized_length() + self.pending_rewards.serialized_length()
    }
}

impl CLTyped for Validator {
    fn cl_type() -> CLType {
        CLType::Any
    }
}


pub struct SignerAndSignature {
    pub public_key: PublicKey,
    pub signature: Bytes,
}

impl FromBytes for SignerAndSignature {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (public_key, remainder) = PublicKey::from_bytes(bytes)?;
        let (signature, remainder) = Bytes::from_bytes(remainder)?;

        Ok((Self { public_key, signature }, remainder))
    }
}

impl ToBytes for SignerAndSignature {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.public_key.to_bytes()?);
        result.extend(self.signature.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.public_key.serialized_length() + self.signature.serialized_length()
    }
}


pub struct AddValidator {
    pub new_validator_public_key: PublicKey,
    pub signatures: Vec<SignerAndSignature>,
}

impl FromBytes for AddValidator {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (new_validator_public_key, remainder) = PublicKey::from_bytes(bytes)?;
        let (signatures, remainder) = Vec::from_bytes(remainder)?;

        Ok((Self { new_validator_public_key, signatures }, remainder))
    }
}

impl ToBytes for AddValidator {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.new_validator_public_key.to_bytes()?);
        result.extend(self.signatures.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.new_validator_public_key.serialized_length() + self.signatures.serialized_length()
    }
}

impl CLTyped for AddValidator {
    fn cl_type() -> CLType {
        CLType::Any
    }
}


pub struct OriginalToDuplicateContractInfo {
    pub chain: String,
    pub contract_address: ContractHash,
}

impl FromBytes for OriginalToDuplicateContractInfo {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (chain, remainder) = String::from_bytes(bytes)?;
        let (contract_address, remainder) = ContractHash::from_bytes(remainder)?;

        Ok((Self { chain, contract_address }, remainder))
    }
}

impl ToBytes for OriginalToDuplicateContractInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.chain.to_bytes()?);
        result.extend(self.contract_address.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.chain.serialized_length() + self.contract_address.serialized_length()
    }
}

impl CLTyped for OriginalToDuplicateContractInfo {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

pub struct DuplicateToOriginalContractInfo {
    pub chain: String,
    pub contract_address: String,
}

impl FromBytes for DuplicateToOriginalContractInfo {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (chain, remainder) = String::from_bytes(bytes)?;
        let (contract_address, remainder) = String::from_bytes(remainder)?;

        Ok((Self { chain, contract_address }, remainder))
    }
}

impl ToBytes for DuplicateToOriginalContractInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.chain.to_bytes()?);
        result.extend(self.contract_address.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.chain.serialized_length() + self.contract_address.serialized_length()
    }
}

impl CLTyped for crate::structs::DuplicateToOriginalContractInfo {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

pub struct ClaimData {
    pub token_id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub destination_user_address: AccountHash,
    pub source_nft_contract_address: String,
    pub name: String,
    pub symbol: String,
    pub royalty: U512,
    pub royalty_receiver: AccountHash,
    pub metadata: String,
    pub transaction_hash: String,
    pub token_amount: U512,
    pub nft_type: String,
    pub fee: U512,
    pub lock_tx_chain: String,
}

impl FromBytes for ClaimData {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (token_id, remainder) = String::from_bytes(bytes)?;
        let (source_chain, remainder) = String::from_bytes(remainder)?;
        let (destination_chain, remainder) = String::from_bytes(bytes)?;
        let (destination_user_address, remainder) = AccountHash::from_bytes(remainder)?;
        let (source_nft_contract_address, remainder) = String::from_bytes(bytes)?;
        let (name, remainder) = String::from_bytes(remainder)?;
        let (symbol, remainder) = String::from_bytes(bytes)?;
        let (royalty, remainder) = U512::from_bytes(remainder)?;
        let (royalty_receiver, remainder) = AccountHash::from_bytes(bytes)?;
        let (metadata, remainder) = String::from_bytes(remainder)?;
        let (transaction_hash, remainder) = String::from_bytes(bytes)?;
        let (token_amount, remainder) = U512::from_bytes(remainder)?;
        let (nft_type, remainder) = String::from_bytes(bytes)?;
        let (fee, remainder) = U512::from_bytes(remainder)?;
        let (lock_tx_chain, remainder) = String::from_bytes(bytes)?;

        Ok((Self {
            token_id,
            source_chain,
            destination_chain,
            destination_user_address,
            source_nft_contract_address,
            name,
            symbol,
            royalty,
            royalty_receiver,
            metadata,
            transaction_hash,
            token_amount,
            nft_type,
            fee,
            lock_tx_chain,
        }, remainder))
    }
}

impl ToBytes for ClaimData {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.token_id.to_bytes()?);
        result.extend(self.source_chain.to_bytes()?);
        result.extend(self.destination_chain.to_bytes()?);
        result.extend(self.destination_user_address.to_bytes()?);
        result.extend(self.source_nft_contract_address.to_bytes()?);
        result.extend(self.name.to_bytes()?);
        result.extend(self.symbol.to_bytes()?);
        result.extend(self.royalty.to_bytes()?);
        result.extend(self.royalty_receiver.to_bytes()?);
        result.extend(self.metadata.to_bytes()?);
        result.extend(self.transaction_hash.to_bytes()?);
        result.extend(self.token_amount.to_bytes()?);
        result.extend(self.nft_type.to_bytes()?);
        result.extend(self.fee.to_bytes()?);
        result.extend(self.lock_tx_chain.to_bytes()?);

        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.token_id.serialized_length() +
            self.source_chain.serialized_length() +
            self.destination_chain.serialized_length() +
            self.destination_user_address.serialized_length() +
            self.source_nft_contract_address.serialized_length() +
            self.name.serialized_length() +
            self.symbol.serialized_length() +
            self.royalty.serialized_length() +
            self.royalty_receiver.serialized_length() +
            self.metadata.serialized_length() +
            self.transaction_hash.serialized_length() +
            self.token_amount.serialized_length() +
            self.nft_type.serialized_length() +
            self.fee.serialized_length() +
            self.lock_tx_chain.serialized_length()
    }
}

impl CLTyped for ClaimData {
    fn cl_type() -> CLType {
        CLType::Any
    }
}