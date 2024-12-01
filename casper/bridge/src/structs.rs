use crate::errors::BridgeError;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use casper_types::account::AccountHash;
use casper_types::bytesrepr::U8_SERIALIZED_LENGTH;
use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, ContractHash, PublicKey, U512,
};
use common::collection::TokenIdentifier;
use core::convert::TryFrom;
// pub type Sigs = (PublicKey, [u8; 64]);
////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Waiting {
    pub wait: bool,
    pub data_type: String,
}
impl FromBytes for Waiting {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (wait, remainder) = bool::from_bytes(bytes)?;
        let (data_type, remainder) = String::from_bytes(remainder)?;

        Ok((Self { wait, data_type }, remainder))
    }
}
impl ToBytes for Waiting {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.wait.to_bytes()?);
        result.extend(self.data_type.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.wait.serialized_length() + self.data_type.serialized_length()
    }
}
impl CLTyped for Waiting {
    fn cl_type() -> CLType {
        CLType::Tuple2([Box::new(CLType::Bool), Box::new(CLType::String)])
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct OtherToken {
    pub token_id: String,
    pub chain: String,
    pub contract_address: String,
}
impl FromBytes for OtherToken {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (token_id, remainder) = String::from_bytes(bytes)?;
        let (chain, remainder) = String::from_bytes(remainder)?;
        let (contract_address, remainder) = String::from_bytes(remainder)?;

        Ok((
            Self {
                token_id,
                chain,
                contract_address,
            },
            remainder,
        ))
    }
}
impl ToBytes for OtherToken {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.token_id.to_bytes()?);
        result.extend(self.chain.to_bytes()?);
        result.extend(self.contract_address.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.token_id.serialized_length()
            + self.chain.serialized_length()
            + self.contract_address.serialized_length()
    }
}
impl CLTyped for OtherToken {
    fn cl_type() -> CLType {
        CLType::Tuple3([
            Box::new(CLType::String),
            Box::new(CLType::String),
            Box::new(CLType::String),
        ])
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SelfToken {
    pub token_id: TokenIdentifier,
    pub chain: String,
    pub contract_address: ContractHash,
}
impl FromBytes for SelfToken {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (token_id, remainder) = TokenIdentifier::from_bytes(bytes)?;
        let (chain, remainder) = String::from_bytes(remainder)?;
        let (contract_address, remainder) = ContractHash::from_bytes(remainder)?;

        Ok((
            Self {
                token_id,
                chain,
                contract_address,
            },
            remainder,
        ))
    }
}
impl ToBytes for SelfToken {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.token_id.to_bytes()?);
        result.extend(self.chain.to_bytes()?);
        result.extend(self.contract_address.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.token_id.serialized_length()
            + self.chain.serialized_length()
            + self.contract_address.serialized_length()
    }
}
impl CLTyped for SelfToken {
    fn cl_type() -> CLType {
        CLType::Tuple3([
            Box::new(CLType::String),
            Box::new(CLType::String),
            Box::new(CLType::ByteArray(32)),
        ])
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Sigs {
    pub public_key: PublicKey,
    pub signature: [u8; 64],
}
impl FromBytes for Sigs {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (public_key, remainder) = PublicKey::from_bytes(bytes)?;
        let (signature, remainder) = <[u8; 64]>::from_bytes(remainder)?;

        Ok((
            Self {
                public_key,
                signature,
            },
            remainder,
        ))
    }
}
impl ToBytes for Sigs {
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
impl CLTyped for Sigs {
    fn cl_type() -> CLType {
        CLType::Tuple2([Box::new(CLType::PublicKey), Box::new(CLType::ByteArray(64))])
    }
}

////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum DataType {
    Claim = 0,
    AddValidator = 1,
    BlacklistValidator = 2,
    ChangeCollectionDeployFee = 3,
    ChangeStorageDeployFee = 4,
}
impl TryFrom<u8> for DataType {
    type Error = BridgeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DataType::Claim),
            1 => Ok(DataType::AddValidator),
            2 => Ok(DataType::BlacklistValidator),
            3 => Ok(DataType::ChangeCollectionDeployFee),
            4 => Ok(DataType::ChangeStorageDeployFee),
            _ => Err(BridgeError::InvalidDataType),
        }
    }
}
impl FromBytes for DataType {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        match bytes.split_first() {
            None => Err(bytesrepr::Error::EarlyEndOfStream),
            Some((byte, rem)) => match DataType::try_from(*byte) {
                Ok(kind) => Ok((kind, rem)),
                Err(_) => Err(bytesrepr::Error::EarlyEndOfStream),
            },
        }
    }
}
impl ToBytes for DataType {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        match self {
            DataType::Claim => {
                result.extend((DataType::Claim as u8).to_bytes()?);
            }
            DataType::AddValidator => {
                result.extend((DataType::AddValidator as u8).to_bytes()?);
            }
            DataType::BlacklistValidator => {
                result.extend((DataType::BlacklistValidator as u8).to_bytes()?);
            }
            DataType::ChangeCollectionDeployFee => {
                result.extend((DataType::ChangeCollectionDeployFee as u8).to_bytes()?);
            }
            DataType::ChangeStorageDeployFee => {
                result.extend((DataType::ChangeStorageDeployFee as u8).to_bytes()?);
            }
        };
        Ok(result)
    }
    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
    }
}
impl CLTyped for DataType {
    fn cl_type() -> CLType {
        CLType::U8
    }
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DoneInfo {
    pub done: bool,
    pub can_do: bool,
    pub fee_taken: bool,
    pub data_type: DataType,
}
impl FromBytes for DoneInfo {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (done, remainder) = bool::from_bytes(bytes)?;
        let (can_do, remainder) = bool::from_bytes(remainder)?;
        let (fee_taken, remainder) = bool::from_bytes(remainder)?;
        let (data_type, remainder) = u8::from_bytes(remainder)?;
        let data_type = DataType::try_from(data_type).map_err(|_| bytesrepr::Error::Formatting)?;

        Ok((
            Self {
                done,
                can_do,
                fee_taken,
                data_type,
            },
            remainder,
        ))
    }
}
impl ToBytes for DoneInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.done.to_bytes()?);
        result.extend(self.can_do.to_bytes()?);
        result.extend(self.fee_taken.to_bytes()?);

        match self.data_type {
            DataType::Claim => {
                result.extend((DataType::Claim as u8).to_bytes()?);
            }
            DataType::AddValidator => {
                result.extend((DataType::AddValidator as u8).to_bytes()?);
            }
            DataType::BlacklistValidator => {
                result.extend((DataType::BlacklistValidator as u8).to_bytes()?);
            }
            DataType::ChangeCollectionDeployFee => {
                result.extend((DataType::ChangeCollectionDeployFee as u8).to_bytes()?);
            }
            DataType::ChangeStorageDeployFee => {
                result.extend((DataType::ChangeStorageDeployFee as u8).to_bytes()?);
            }
        };

        Ok(result)
    }
    fn serialized_length(&self) -> usize {
        self.done.serialized_length()
            + self.can_do.serialized_length()
            + self.fee_taken.serialized_length()
            + U8_SERIALIZED_LENGTH
    }
}
impl CLTyped for DoneInfo {
    fn cl_type() -> CLType {
        CLType::Tuple2([
            Box::new(CLType::Tuple3([
                Box::new(CLType::Bool),
                Box::new(CLType::Bool),
                Box::new(CLType::Bool),
            ])),
            Box::new(CLType::Tuple1([Box::new(CLType::U8)])),
        ])
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Receiver {
    Sender,
    Service,
    Bridge,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Validator {
    pub added: bool,
    pub pending_rewards: U512,
}
impl FromBytes for Validator {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (added, remainder) = bool::from_bytes(bytes)?;
        let (pending_rewards, remainder) = U512::from_bytes(remainder)?;

        Ok((
            Self {
                added,
                pending_rewards,
            },
            remainder,
        ))
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
        CLType::Tuple2([Box::new(CLType::Bool), Box::new(CLType::U512)])
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

        Ok((
            Self {
                chain,
                contract_address,
            },
            remainder,
        ))
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
        CLType::Tuple2([Box::new(CLType::String), Box::new(CLType::ByteArray(32))])
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

        Ok((
            Self {
                chain,
                contract_address,
            },
            remainder,
        ))
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
impl CLTyped for DuplicateToOriginalContractInfo {
    fn cl_type() -> CLType {
        CLType::Tuple2([Box::new(CLType::String), Box::new(CLType::String)])
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
        let (destination_chain, remainder) = String::from_bytes(remainder)?;
        let (destination_user_address, remainder) = AccountHash::from_bytes(remainder)?;
        let (source_nft_contract_address, remainder) = String::from_bytes(remainder)?;
        let (name, remainder) = String::from_bytes(remainder)?;
        let (symbol, remainder) = String::from_bytes(remainder)?;
        let (royalty, remainder) = U512::from_bytes(remainder)?;
        let (royalty_receiver, remainder) = AccountHash::from_bytes(remainder)?;
        let (metadata, remainder) = String::from_bytes(remainder)?;
        let (transaction_hash, remainder) = String::from_bytes(remainder)?;
        let (token_amount, remainder) = U512::from_bytes(remainder)?;
        let (nft_type, remainder) = String::from_bytes(remainder)?;
        let (fee, remainder) = U512::from_bytes(remainder)?;
        let (lock_tx_chain, remainder) = String::from_bytes(remainder)?;

        Ok((
            Self {
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
            },
            remainder,
        ))
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
        self.token_id.serialized_length()
            + self.source_chain.serialized_length()
            + self.destination_chain.serialized_length()
            + self.destination_user_address.serialized_length()
            + self.source_nft_contract_address.serialized_length()
            + self.name.serialized_length()
            + self.symbol.serialized_length()
            + self.royalty.serialized_length()
            + self.royalty_receiver.serialized_length()
            + self.metadata.serialized_length()
            + self.transaction_hash.serialized_length()
            + self.token_amount.serialized_length()
            + self.nft_type.serialized_length()
            + self.fee.serialized_length()
            + self.lock_tx_chain.serialized_length()
    }
}
impl CLTyped for ClaimData {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
