use alloc::{string::String, vec::Vec};
use casper_types::{
    account::AccountHash, bytesrepr::{self, Bytes, FromBytes, ToBytes}, CLType, CLTyped, ContractHash, Key, U256, U512, PublicKey
};

pub struct Validator {
    pub added: bool,
    pub pending_rewards: U256,
}

impl FromBytes for Validator {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {

        let (added, remainder) = bool::from_bytes(bytes)?;
        let (pending_rewards, remainder) = U256::from_bytes(remainder)?;

        Ok((Self { added,pending_rewards }, remainder))
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
