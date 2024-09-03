use near_sdk::ext_contract;
use nft::metadata::NFTContractMetadata;

#[allow(dead_code)]
#[ext_contract(collection)]
pub trait NFT {
    fn new(owner_id: near_sdk::AccountId, metadata: NFTContractMetadata);
}
