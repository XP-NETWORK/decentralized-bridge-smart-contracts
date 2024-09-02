use near_sdk::ext_contract;

#[allow(dead_code)]
#[ext_contract(storage)]
pub trait Storage {
    fn new(
        owner: near_sdk::AccountId,
        collection: near_sdk::AccountId,
    );
}