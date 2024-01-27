use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Bridge {
    pub validator_count: u64,
    pub current_token_id: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Validators {
    pub added: bool,
    pub pending_rewards: u128,
}

#[account]
#[derive(InitSpace)]
pub struct SignatureThreshold {
    pub threshold: u64,
}

#[account]
#[derive(InitSpace)]
pub struct ContractInfo {
    #[max_len(20)]
    pub chain: String,
    #[max_len(50)]
    pub contract_address: String,
}

#[account]
#[derive(InitSpace)]
pub struct SelfTokenInfo {
    pub token_id: Pubkey,
    #[max_len(20)]
    pub chain: String,
    #[max_len(50)]
    pub contract_address: String,
}

#[account]
#[derive(InitSpace)]
pub struct OtherTokenInfo {
    #[max_len(50)]
    pub token_id: String,
    #[max_len(20)]
    pub chain: String,
    #[max_len(50)]
    pub contract_address: String,
}
