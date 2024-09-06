use multiversx_sc::{
    api::ManagedTypeApi,
    types::{BigUint, ManagedAddress, ManagedBuffer},
};

multiversx_sc::derive_imports!();

#[derive(Clone, NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem)]
pub struct Validator<M: ManagedTypeApi> {
    pub added: bool,
    pub pending_reward: BigUint<M>,
}

#[derive(Clone, NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem)]
pub struct TokenInfo<M: ManagedTypeApi> {
    pub token_id: u64,
    pub chain: ManagedBuffer<M>,
    pub contract_address: ManagedBuffer<M>,
}

impl<M> Default for TokenInfo<M>
where
    M: ManagedTypeApi,
{
    fn default() -> Self {
        Self {
            token_id: Default::default(),
            chain: Default::default(),
            contract_address: Default::default(),
        }
    }
}

impl<M> PartialEq for TokenInfo<M>
where
    M: ManagedTypeApi,
{
    fn eq(&self, other: &Self) -> bool {
        self.token_id == other.token_id
            && self.chain == other.chain
            && self.contract_address == other.contract_address
    }
}

#[derive(Clone, NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem)]
pub struct SignatureInfo<M: ManagedTypeApi> {
    pub public_key: ManagedAddress<M>,
    pub sig: ManagedBuffer<M>,
}

#[derive(Clone, NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem)]
pub struct ContractInfo<M: ManagedTypeApi> {
    pub chain: ManagedBuffer<M>,
    pub address: ManagedBuffer<M>,
}

#[derive(Clone, NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem)]
pub struct ClaimData<M: ManagedTypeApi> {
    pub token_id: ManagedBuffer<M>,
    pub source_chain: ManagedBuffer<M>,
    pub destination_chain: ManagedBuffer<M>,
    pub destination_user_address: ManagedAddress<M>,
    pub source_nft_contract_address: ManagedBuffer<M>,
    pub name: ManagedBuffer<M>,
    pub symbol: ManagedBuffer<M>,
    pub royalty: BigUint<M>,
    pub royalty_receiver: ManagedAddress<M>,
    pub attrs: ManagedBuffer<M>,
    pub transaction_hash: ManagedBuffer<M>,
    pub token_amount: BigUint<M>,
    pub nft_type: ManagedBuffer<M>,
    pub fee: BigUint<M>,
    pub lock_tx_chain: ManagedBuffer<M>,
    pub img_uri: ManagedBuffer<M>
}
