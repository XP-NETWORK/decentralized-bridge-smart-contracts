use near_sdk::ext_contract;

#[allow(dead_code)]
#[ext_contract(common_nft)]
pub trait CommonNft {
    fn nft_transfer(
        &mut self,
        receiver_id: near_sdk::AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
}