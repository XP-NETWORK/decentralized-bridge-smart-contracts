pub mod error;
pub mod state;

use anchor_lang::{prelude::*, solana_program::{hash::hash, sysvar::SysvarId ,system_instruction}};
use anchor_spl::{
    self,
    token::{self, Mint, Token, TokenAccount, Transfer}, associated_token::AssociatedToken, metadata::Metadata,
};
use mpl_token_metadata::pda::{find_metadata_account, find_master_edition_account};
use state::{Bridge, ContractInfo, OtherTokenInfo, SelfTokenInfo, SignatureThreshold, Validators};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use anchor_lang::solana_program::ed25519_program::ID as ED25519_ID;
use anchor_lang::solana_program::program::invoke;

declare_id!("5gyedJbp5cuECB3K7Z4axGe9UkA2McxwndjTw8D4EXgX");

#[program]
pub mod xp_bridge {
    pub const SELF_CHAIN: &str = "SOL";
    pub const TYPE_NFT: &str = "nft"; // a more general term to accomodate non-evm chains
    pub const TYPE_SFT: &str = "sft"; // a more general term to accomodate non-evm chains
    pub const BRIDGE: &str = "bridge";
    pub const VALIDATOR: &str = "validator";
    pub const THRESHOLD: &str = "threshold";
    pub const REWARD: &str = "reward";
    pub const SELF_TOKENS: &str = "self_tokens";
    pub const OTHER_TOKENS: &str = "other_tokens";
    pub const ORIGINAL_TO_DUPLICATE_MAPPING: &str = "original_to_duplicate_mapping";
    pub const DUPLICATE_TO_ORIGINAL_MAPPING: &str = "duplicate_to_original_mapping";
    pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _data: InitializeData) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.validator_count += 1;
        bridge.current_token_id += 1;

        let validator = &mut ctx.accounts.validators;
        validator.added = true;
        validator.pending_rewards = 0;

        Ok(())
    }

    pub fn verify_add_validator_signatures(ctx: Context<VerifyAddValidatorSignatures>, data: VerifyAddValidatorSignaturesData) -> Result<()> {
        if data.signatures.len() == 0 {
            return Err(error::ErrorCode::NoSignatures.into());
        }
        let threshold_account = &mut ctx.accounts.threshold;
        let mut percentage: u64 = 0;

        for (index,sig_info) in data.signatures.into_iter().enumerate() {

            let ix: Instruction = load_instruction_at_checked(index, &ctx.accounts.instruction_acc)?;
            //Check that ix is what we expect to have been sent
            let res = utils::verify_ed25519_ix(&ix, &sig_info.public_key.to_bytes(), &data.validator_public_key.to_bytes(), &sig_info.sig)?;

            if res {
                let validator_account: Result<Account<Validators>> = Account::try_from(&ctx.remaining_accounts[index]);
                match validator_account {
                    Ok(v)=>{
                        if v.added {
                            percentage += 1;
                        }
                    },
                    Err(_) => todo!()
                }
            }
        }
        threshold_account.threshold += percentage;

        Ok(())
    }

    pub fn verify_claim_signatures(ctx: Context<VerifyClaimSignatures>, data: VerifyClaimSignaturesData) -> Result<()> {
        if data.signatures.len() == 0 {
            return Err(error::ErrorCode::NoSignatures.into());
        }
        let threshold_account = &mut ctx.accounts.threshold;
        let mut percentage: u64 = 0;
        let claim_data = data.claim_data.try_to_vec()?;
        let total_rewards = ctx.accounts.bridge.to_account_info().lamports();
        let fee = data.claim_data.fee;

        if total_rewards < fee {
            return Err(error::ErrorCode::NoRewardsAvailable.into());
        }

        let fee_per_validator = fee / data.validators_count;

        for (index,sig_info) in data.signatures.into_iter().enumerate() {

            let ix: Instruction = load_instruction_at_checked(index, &ctx.accounts.instruction_acc)?;

            //Check that ix is what we expect to have been sent
            let res = utils::verify_ed25519_ix(&ix, &sig_info.public_key.to_bytes(), &claim_data, &sig_info.sig)?;

            if res {
                let validator_account: Result<Account<Validators>> = Account::try_from(&ctx.remaining_accounts[index]);
                match validator_account {
                    Ok(mut v)=>{
                        if v.added {
                            percentage += 1;
                            v.pending_rewards += fee_per_validator;
                        }
                    },
                    Err(_) => todo!()
                }
            }
        }
        threshold_account.threshold += percentage;
        
        Ok(())
    }

    pub fn verify_reward_validator_signatures(ctx: Context<VerifyRewardValidatorSignatures>, data: VerifyRewardValidatorSignaturesData) -> Result<()> {
        if data.signatures.len() == 0 {
            return Err(error::ErrorCode::NoSignatures.into());
        }
        let threshold_account = &mut ctx.accounts.threshold;
        let mut percentage: u64 = 0;

        for (index,sig_info) in data.signatures.into_iter().enumerate() {

            let ix: Instruction = load_instruction_at_checked(index, &ctx.accounts.instruction_acc)?;
            //Check that ix is what we expect to have been sent
            let res = utils::verify_ed25519_ix(&ix, &sig_info.public_key.to_bytes(), &data.validator_public_key.to_bytes(), &sig_info.sig)?;

            if res {
                let validator_account: Result<Account<Validators>> = Account::try_from(&ctx.remaining_accounts[index]);
                match validator_account {
                    Ok(v)=>{
                        if v.added {
                            percentage += 1;
                        }
                    },
                    Err(_) => todo!()
                }
            }
        }
        threshold_account.threshold += percentage;

        Ok(())
    }

    pub fn add_validator(ctx: Context<AddValidator>, _data: AddValidatorData) -> Result<()> {
        
        let validators_count = ctx.accounts.bridge.validator_count;
        let percentage = ctx.accounts.threshold.threshold;

        if !(percentage >= (((validators_count * 2) / 3) + 1)) {
            return Err(error::ErrorCode::NoSignatures.into())
        }

        let bridge = &mut ctx.accounts.bridge;
        bridge.validator_count += 1;

        let validator = &mut ctx.accounts.validators;
        validator.added = true;
        validator.pending_rewards = 0;
        Ok(())
    }

    pub fn claim_validator_rewards(ctx: Context<ValidatorClaimReward>, _data: ValidatorClaimRewardData) -> Result<()> {
        let validators_count = ctx.accounts.bridge.validator_count;
        let percentage = ctx.accounts.threshold.threshold;

        if !(percentage >= (((validators_count * 2) / 3) + 1)) {
            return Err(error::ErrorCode::NoSignatures.into())
        }

        let from_account = &ctx.accounts.bridge.to_account_info();
        let to_account = &ctx.accounts.validators.to_account_info();

        let bridge_balance = **from_account.try_borrow_lamports()?;
        let min_balance = LAMPORTS_PER_SOL / 100;

        if bridge_balance <= min_balance {
            return err!(error::ErrorCode::NoRewardsAvailable);
        }

        let amount = bridge_balance - min_balance; // current balance of bridge - minimum balance of bridge
        
         // Does the from account have enough lamports to transfer?
        if **from_account.try_borrow_lamports()? < amount {
            return err!(error::ErrorCode::NoRewardsAvailable);
        }
        // Debit from_account and credit to_account
        **from_account.try_borrow_mut_lamports()? -= amount;
        **to_account.try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    pub fn lock_nft(ctx: Context<Lock>, data: LockData) -> Result<()> {

        if data.token_amount != 1 {
            return Err(error::ErrorCode::InvalidTokenAmount.into());
        }

        let original_collection_address = &mut ctx.accounts.duplicate_to_original_mapping;
        let other_tokens =&mut ctx.accounts.other_tokens;
        let self_tokens =&mut ctx.accounts.self_tokens;
        let bridge = &mut ctx.accounts.bridge;

        let is_token_exists;

        if other_tokens.chain != ""  {
            is_token_exists = Some(other_tokens.clone());
        }
        else{
            is_token_exists = Option::None;
        }
        

        let mut_token_id: String;

        match is_token_exists {
            Some(v)=>{
                mut_token_id = v.token_id.to_string();
            },
            None=>{
                mut_token_id = bridge.current_token_id.to_string();
                other_tokens.token_id = bridge.current_token_id.to_string();
                other_tokens.chain = data.destination_chain.to_string();
                other_tokens.contract_address = data.source_nft_contract_address.to_string();

                self_tokens.token_id = data.token_id;
                self_tokens.chain = SELF_CHAIN.to_string();
                self_tokens.contract_address = data.source_nft_contract_address.to_string();
            }
        }

        let transfer_ctx = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_ctx),
            1,
        )?;
        if original_collection_address.chain != "" {
            emit!(LockEvent { 
                token_id: mut_token_id, 
                destination_chain: data.destination_chain, 
                destination_user_address: data.destination_user_address, 
                source_nft_contract_address: ctx.accounts.duplicate_to_original_mapping.contract_address.clone(), 
                token_amount: data.token_amount, 
                nft_type: "nft".to_string(), 
                chain: ctx.accounts.duplicate_to_original_mapping.chain.clone()});
        }
        else{
            emit!(LockEvent { 
                token_id: mut_token_id, 
                destination_chain: data.destination_chain, 
                destination_user_address: data.destination_user_address, 
                source_nft_contract_address: data.source_nft_contract_address.to_string(), 
                token_amount: data.token_amount, 
                nft_type: "nft".to_string(), 
                chain: SELF_CHAIN.to_string()});
        } 
        Ok(())
    }

    pub fn lock_sft(ctx: Context<Lock>, data: LockData) -> Result<()> {
        if data.token_amount > 0 {
            return Err(error::ErrorCode::InvalidTokenAmount.into());
        }
        let original_collection_address = &mut ctx.accounts.duplicate_to_original_mapping;
        let other_tokens =&mut ctx.accounts.other_tokens;
        let self_tokens =&mut ctx.accounts.self_tokens;

        let is_token_exists;

        if other_tokens.chain != ""  {
            is_token_exists = Some(other_tokens.clone());
        }
        else{
            is_token_exists = Option::None;
        }
        
        let mut_token_id: String;

        match is_token_exists {
            Some(v)=>{
                mut_token_id = v.token_id.to_string();
            },
            None=>{
                mut_token_id = data.token_id.to_string();
                other_tokens.token_id = data.token_id.to_string();
                other_tokens.chain = "SOL".to_string();
                other_tokens.contract_address = data.source_nft_contract_address.to_string();

                self_tokens.token_id = data.token_id;
                self_tokens.chain = "SOL".to_string();
                self_tokens.contract_address = data.source_nft_contract_address.to_string();
            }
        }

        let transfer_ctx = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_ctx),
            data.token_amount,
        )?;
        if original_collection_address.chain != "" {
            emit!(LockEvent { 
                token_id: mut_token_id, 
                destination_chain: data.destination_chain, 
                destination_user_address: data.destination_user_address, 
                source_nft_contract_address: ctx.accounts.duplicate_to_original_mapping.contract_address.clone(), 
                token_amount: data.token_amount, 
                nft_type: "sft".to_string(), 
                chain: ctx.accounts.duplicate_to_original_mapping.chain.clone()});
        }
        else{
            emit!(LockEvent { 
                token_id: mut_token_id, 
                destination_chain: data.destination_chain, 
                destination_user_address: data.destination_user_address, 
                source_nft_contract_address: data.source_nft_contract_address.to_string(), 
                token_amount: data.token_amount, 
                nft_type: "sft".to_string(), 
                chain: "SOL".to_string()});
        } 
        
        Ok(())
    }

    pub fn claim_nft_with_collection_creation(ctx: Context<ClaimCreateCollection>, data: ClaimData) -> Result<()> {
        if !data.claim_data.nft_type.as_bytes().eq(TYPE_NFT.as_bytes()) {
            return Err(error::ErrorCode::InvalidNft.into());
        }


        let validators_count = ctx.accounts.bridge.validator_count;
        let percentage = ctx.accounts.threshold.threshold;

        if !(percentage >= (((validators_count * 2) / 3) + 1)) {
            return Err(error::ErrorCode::NoSignatures.into())
        }

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.user.to_account_info().key(),
                &ctx.accounts.bridge.to_account_info().key(),
                data.claim_data.fee,
            ),
            &[
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.bridge.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;

        let binding = hash([data.claim_data.source_nft_contract_address.clone(),data.claim_data.source_chain.clone()].concat().as_bytes());
        let collection_seed_hash = binding.as_ref();
        let duplicate_collection_address = &mut ctx.accounts.original_to_duplicate_mapping;
        let self_tokens =&mut ctx.accounts.self_tokens;

        let is_token_exists;
        msg!("asdfasd {}",duplicate_collection_address.chain);

            if self_tokens.chain != ""  {
                msg!("is_token_exists true");
                is_token_exists = Some(self_tokens);
            }
            else{
                msg!("is_token_exists false");
                is_token_exists = Option::None;
            }
    
        if duplicate_collection_address.chain != "" {
           msg!("LORA 6")
        }
        else{
            // notDuplicate
            match is_token_exists {
                Some(_v)=>{
                    msg!("LORA 7")
                },
                None=>{
                    let _ = collection_createor::create_collection_nft(collection_seed_hash, &ctx, data.claim_data.metadata.clone(), data.claim_data.name.clone(), data.claim_data.symbol.clone());

                    let _ = collection_createor::create_nft_in_collection(collection_seed_hash, &ctx, data.claim_data.metadata, data.claim_data.name, data.claim_data.symbol, data.claim_data.token_amount);
                    msg!("gandddododo");
                    let otdm = &mut ctx.accounts.original_to_duplicate_mapping;
                    let dtom = &mut ctx.accounts.duplicate_to_original_mapping;


                    let other_tokens = &mut ctx.accounts.other_tokens;
                    let self_tokens = &mut ctx.accounts.self_tokens;

                    otdm.chain = SELF_CHAIN.to_string();
                    otdm.contract_address = ctx.accounts.create_collection_mint.key().to_string();

                    dtom.chain = data.claim_data.source_chain.clone();
                    dtom.contract_address = data.claim_data.source_nft_contract_address.clone();

                    other_tokens.token_id = data.claim_data.token_id;
                    other_tokens.chain = data.claim_data.source_chain.clone();
                    other_tokens.contract_address = data.claim_data.source_nft_contract_address.clone();

                    self_tokens.token_id = ctx.accounts.nft_mint.key();
                    self_tokens.chain = SELF_CHAIN.to_string();
                    self_tokens.contract_address = ctx.accounts.create_collection_mint.key().to_string();

                    // let transfer_ctx = Transfer {
                    //     from: ctx.accounts.create_collection_mint.to_account_info(),
                    //     to: ctx.accounts.nft_token_account.to_account_info(),
                    //     authority: ctx.accounts.create_collection_mint.to_account_info(),
                    // };
                    
                    // let auth_seeds = [b"collection".as_ref(), &[0]];
                    // msg!("gandddododo 2");
                    // token::transfer(
                    //     CpiContext::new_with_signer(
                    //         ctx.accounts.token_program.to_account_info(), 
                    //         transfer_ctx,
                    //         &[&auth_seeds]
                    //     ),
                    //     1,
                    // )?;
                    // msg!("gandddododo 3");
                }
            }
        }
        emit!(ClaimEvent{
            source_chain: data.claim_data.source_chain,
            transaction_hash: data.claim_data.transaction_hash
        });
        Ok(())
    }

    pub fn claim_nft_with_creation(ctx: Context<ClaimCreateNft>, data: ClaimData) -> Result<()> {
        if !data.claim_data.nft_type.as_bytes().eq(TYPE_NFT.as_bytes()) {
            return Err(error::ErrorCode::InvalidNft.into());
        }

        let validators_count = ctx.accounts.bridge.validator_count;
        let percentage = ctx.accounts.threshold.threshold;

        if !(percentage >= (((validators_count * 2) / 3) + 1)) {
            return Err(error::ErrorCode::NoSignatures.into())
        }

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.user.to_account_info().key(),
                &ctx.accounts.bridge.to_account_info().key(),
                data.claim_data.fee,
            ),
            &[
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.bridge.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;

        let binding = hash([data.claim_data.source_nft_contract_address.clone(),data.claim_data.source_chain.clone()].concat().as_bytes());
        let collection_seed_hash = binding.as_ref();
        let duplicate_collection_address = &mut ctx.accounts.original_to_duplicate_mapping;
        let self_tokens =&mut ctx.accounts.self_tokens;

        let is_token_exists;
        msg!("asdfasd {}",duplicate_collection_address.chain);

            if self_tokens.chain != ""  {
                msg!("is_token_exists true");
                is_token_exists = Some(self_tokens);
            }
            else{
                msg!("is_token_exists false");
                is_token_exists = Option::None;
            }
    
        if duplicate_collection_address.chain != "" {
            // isDuplicate
            match is_token_exists {
                Some(_v)=>{
                    msg!("LORA 3")
                },
                None=>{
                    let _ = collection_createor::create_nft(collection_seed_hash, &ctx, data.claim_data.metadata, data.claim_data.name, data.claim_data.symbol, data.claim_data.token_amount);

                    // let transfer_ctx = Transfer {
                    //     from: ctx.accounts.create_collection_mint.to_account_info(),
                    //     to: ctx.accounts.nft_token_account.to_account_info(),
                    //     authority: ctx.accounts.create_collection_mint.to_account_info(),
                    // };
                    // let auth_seeds = [collection_seed_hash, &[*ctx.bumps.get("create_collection_mint").unwrap()]];

                    // token::transfer(
                    //     CpiContext::new_with_signer(
                    //         ctx.accounts.token_program.to_account_info(), 
                    //         transfer_ctx,
                    //         &[&auth_seeds]
                    //     ),
                    //     1,
                    // )?;

                    let other_tokens = &mut ctx.accounts.other_tokens;
                    let self_tokens = &mut ctx.accounts.self_tokens;

                    other_tokens.token_id = data.claim_data.token_id;
                    other_tokens.chain = data.claim_data.source_chain.clone();
                    other_tokens.contract_address = data.claim_data.source_nft_contract_address.clone();

                    self_tokens.token_id = ctx.accounts.nft_mint.key();
                    self_tokens.chain = SELF_CHAIN.to_string();
                    self_tokens.contract_address = ctx.accounts.create_collection_mint.key().to_string();
                }
            }
        }
        else{
           msg!("LORA 5");
        }
        emit!(ClaimEvent{
            source_chain: data.claim_data.source_chain,
            transaction_hash: data.claim_data.transaction_hash
        });
        Ok(())
    }

    pub fn claim_nft_just_unlock(ctx: Context<ClaimUnlock>, data: ClaimData) -> Result<()> {
        if !data.claim_data.nft_type.as_bytes().eq(TYPE_NFT.as_bytes()) {
            return Err(error::ErrorCode::InvalidNft.into());
        }

        let validators_count = ctx.accounts.bridge.validator_count;
        let percentage = ctx.accounts.threshold.threshold;

        if !(percentage >= (((validators_count * 2) / 3) + 1)) {
            return Err(error::ErrorCode::NoSignatures.into())
        }

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.user.to_account_info().key(),
                &ctx.accounts.bridge.to_account_info().key(),
                data.claim_data.fee,
            ),
            &[
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.bridge.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;

        let duplicate_collection_address = &mut ctx.accounts.original_to_duplicate_mapping;
        let self_tokens =&mut ctx.accounts.self_tokens;

        let is_token_exists;
        msg!("asdfasd {}",duplicate_collection_address.chain);

            if self_tokens.chain != ""  {
                msg!("is_token_exists true");
                is_token_exists = Some(self_tokens);
            }
            else{
                msg!("is_token_exists false");
                is_token_exists = Option::None;
            }
    
        if duplicate_collection_address.chain != "" {
            // isDuplicate
            match is_token_exists {
                Some(_v)=>{
                    // Unlock721
                    let transfer_ctx = Transfer {
                        from: ctx.accounts.bridge_token_account.to_account_info(),
                        to: ctx.accounts.nft_token_account.to_account_info(),
                        authority: ctx.accounts.bridge.to_account_info(),
                    };
                    let auth_seeds = [BRIDGE.as_bytes(), &[*ctx.bumps.get("bridge").unwrap()]];
                    msg!("popopopopo");
                    token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(), 
                            transfer_ctx,
                            &[&auth_seeds]
                        ),
                        1,
                    )?;
                },
                None=>{
                    msg!("LORA");
                }
            }
        }
        else{
            // notDuplicate
            match is_token_exists {
                Some(_v)=>{
                    // Unlock721
                    let transfer_ctx = Transfer {
                        from: ctx.accounts.bridge_token_account.to_account_info(),
                        to: ctx.accounts.nft_token_account.to_account_info(),
                        authority: ctx.accounts.bridge.to_account_info(),
                    };
                    let auth_seeds = [BRIDGE.as_bytes(), &[*ctx.bumps.get("bridge").unwrap()]];
                    msg!("popopopopsssssssssso");
                    token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(), 
                            transfer_ctx,
                            &[&auth_seeds]
                        ),
                        1,
                    )?;
                },
                None=>{
                    msg!("LORA 2")
                }
            }
        }
        emit!(ClaimEvent{
            source_chain: data.claim_data.source_chain,
            transaction_hash: data.claim_data.transaction_hash
        });
        Ok(())
    }
    
}

pub mod utils {
    use anchor_lang::solana_program::{self};

    use super::*;
    /// Verify Ed25519Program instruction fields
    pub fn verify_ed25519_ix(ix: &Instruction, pubkey: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool> {
        msg!("here verify_ed25519_ix {} {} {} {}",ix.program_id ,ix.accounts.len(),ix.data.len(),16 + 64 + 32 + msg.len());
        if  ix.program_id       != ED25519_ID                   ||  // The program id we expect
            ix.accounts.len()   != 0                              // With no context accounts
        {
            return Err(error::ErrorCode::InvalidSignature.into());    // Otherwise, we can already throw err
        }

        check_ed25519_data(&ix.data, pubkey, msg, sig)?;            // If that's not the case, check data

        Ok(true)
    }

    /// Verify serialized Ed25519Program instruction data
    pub fn check_ed25519_data(data: &[u8], pubkey: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool> {
        msg!("here check_ed25519_data");
        // According to this layout used by the Ed25519Program
        // https://github.com/solana-labs/solana-web3.js/blob/master/src/ed25519-program.ts#L33

        // "Deserializing" byte slices

        let num_signatures                  = &[data[0]];        // Byte  0
        let padding                         = &[data[1]];        // Byte  1
        let signature_offset                = &data[2..=3];      // Bytes 2,3
        let signature_instruction_index     = &data[4..=5];      // Bytes 4,5
        let public_key_offset               = &data[6..=7];      // Bytes 6,7
        let public_key_instruction_index    = &data[8..=9];      // Bytes 8,9
        let message_data_offset             = &data[10..=11];    // Bytes 10,11
        // let message_data_size               = &data[12..=13];    // Bytes 12,13
        let message_instruction_index       = &data[14..=15];    // Bytes 14,15

        let data_pubkey                     = &data[16..16+32];  // Bytes 16..16+32
        let data_sig                        = &data[48..48+64];  // Bytes 48..48+64
        let data_msg                        = &data[112..];      // Bytes 112..end

        // Expected values

        let exp_public_key_offset:      u16 = 16; // 2*u8 + 7*u16
        let exp_signature_offset:       u16 = exp_public_key_offset + pubkey.len() as u16;
        let exp_message_data_offset:    u16 = exp_signature_offset + sig.len() as u16;
        let exp_num_signatures:          u8 = 1;
        // let exp_message_data_size:      u16 = msg.len().try_into().unwrap();

        // Header and Arg Checks

        // Header
        if  num_signatures                  != &exp_num_signatures.to_le_bytes()        ||
            padding                         != &[0]                                     ||
            signature_offset                != &exp_signature_offset.to_le_bytes()      ||
            signature_instruction_index     != &u16::MAX.to_le_bytes()                  ||
            public_key_offset               != &exp_public_key_offset.to_le_bytes()     ||
            public_key_instruction_index    != &u16::MAX.to_le_bytes()                  ||
            message_data_offset             != &exp_message_data_offset.to_le_bytes()   ||
            message_instruction_index       != &u16::MAX.to_le_bytes()  
        {
            return Err(error::ErrorCode::InvalidSignature.into());
        }
        let msg_hash = solana_program::hash::hash(&msg);
        let data_hash = data_msg.to_vec();

        msg!("{:?}", msg_hash.try_to_vec()?);
        msg!("{:?}", data_hash);

        // Arguments
        if  data_pubkey != pubkey   ||
            data_hash   != msg_hash.try_to_vec()?      ||
            data_sig    != sig
        {
            return Err(error::ErrorCode::InvalidSignature.into());
        }

        Ok(true)
    }
}

pub mod collection_createor{
    use anchor_spl::{
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_sized_collection_item, sign_metadata, CreateMasterEditionV3,
        CreateMetadataAccountsV3, SetAndVerifySizedCollectionItem, SignMetadata,
    },
    token::{mint_to, MintTo},
    };
    use mpl_token_metadata::
    state::{CollectionDetails, Creator, DataV2};
    use super::*;
    
    pub fn create_collection_nft(
        seed: &[u8],
        ctx: &Context<ClaimCreateCollection>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        // PDA for signing
        let signer_seeds: &[&[&[u8]]] = &[&[
            seed,
            &[*ctx.bumps.get("create_collection_mint").unwrap()],
        ]];
        msg!(" LO collection start");
        // mint collection nft
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.create_collection_mint.to_account_info(),
                    to: ctx.accounts.create_collection_token_account.to_account_info(),
                    authority: ctx.accounts.create_collection_mint.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        // create metadata account for collection nft
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.create_collection_metadata_account.to_account_info(),
                    mint: ctx.accounts.create_collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(), // use pda mint address as mint authority
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(), // use pda mint as update authority
                    payer: ctx.accounts.user.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: Some(vec![Creator {
                    address: ctx.accounts.user.key(),
                    verified: false,
                    share: 100,
                }]),
                collection: None,
                uses: None,
            },
            true,
            true,
            Some(CollectionDetails::V1 { size: 0 }), // set as collection nft
        )?;

        // create master edition account for collection nft
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.create_collection_mint.to_account_info(),
                    edition: ctx.accounts.create_collection_master_edition.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    metadata: ctx.accounts.create_collection_metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;

        // verify creator on metadata account
        sign_metadata(CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            SignMetadata {
                creator: ctx.accounts.user.to_account_info(),
                metadata: ctx.accounts.create_collection_metadata_account.to_account_info(),
            },
        ))?;
        msg!(" LO collection end");
        Ok(())
    }

    pub fn create_nft_in_collection(
        seed: &[u8],
        ctx: &Context<ClaimCreateCollection>,
        uri: String,
        name: String,
        symbol: String,
        amount: u64,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            seed,
            &[*ctx.bumps.get("create_collection_mint").unwrap()],
        ]];
        msg!(" LO nft start");
        // mint nft in collection
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.nft_token_account.to_account_info(),
                    authority: ctx.accounts.create_collection_mint.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        msg!(" LO nft mid");
        // create metadata account for nft in collection
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.nft_metadata_account.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    payer: ctx.accounts.user.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;

        // create master edition account for nft in collection
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    edition: ctx.accounts.nft_master_edition.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    metadata: ctx.accounts.nft_metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;

        // verify nft as part of collection
        set_and_verify_sized_collection_item(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                SetAndVerifySizedCollectionItem {
                    metadata: ctx.accounts.nft_metadata_account.to_account_info(),
                    collection_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    payer: ctx.accounts.user.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    collection_mint: ctx.accounts.create_collection_mint.to_account_info(),
                    collection_metadata: ctx.accounts.create_collection_metadata_account.to_account_info(),
                    collection_master_edition: ctx
                        .accounts
                        .create_collection_master_edition
                        .to_account_info(),
                },
                &signer_seeds,
            ),
            None,
        )?;
        msg!(" LO nft end");

        Ok(())
    }

    pub fn create_nft(
        seed: &[u8],
        ctx: &Context<ClaimCreateNft>,
        uri: String,
        name: String,
        symbol: String,
        amount: u64,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            seed,
            &[*ctx.bumps.get("create_collection_mint").unwrap()],
        ]];
        msg!(" LO nft start");
        // mint nft in collection
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.nft_token_account.to_account_info(),
                    authority: ctx.accounts.create_collection_mint.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        msg!(" LO nft mid");
        // create metadata account for nft in collection
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.nft_metadata_account.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    payer: ctx.accounts.user.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;

        // create master edition account for nft in collection
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    edition: ctx.accounts.nft_master_edition.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    metadata: ctx.accounts.nft_metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;

        // verify nft as part of collection
        set_and_verify_sized_collection_item(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                SetAndVerifySizedCollectionItem {
                    metadata: ctx.accounts.nft_metadata_account.to_account_info(),
                    collection_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    payer: ctx.accounts.user.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    collection_mint: ctx.accounts.create_collection_mint.to_account_info(),
                    collection_metadata: ctx.accounts.create_collection_metadata_account.to_account_info(),
                    collection_master_edition: ctx
                        .accounts
                        .create_collection_master_edition
                        .to_account_info(),
                },
                &signer_seeds,
            ),
            None,
        )?;
        msg!(" LO nft end");

        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(data: VerifyAddValidatorSignaturesData)]
pub struct VerifyAddValidatorSignatures<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + SignatureThreshold::INIT_SPACE,
        seeds = [hash([THRESHOLD, &data.validator_public_key.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,

     /// CHECK: used to get instruction data
     #[account(address = Instructions::id())]
     pub instruction_acc: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VerifyAddValidatorSignaturesData {
    validator_public_key: Pubkey,
    signatures: Vec<SignatureInfo>,
}

#[derive(Accounts)]
#[instruction(data: VerifyClaimSignaturesData)]
pub struct VerifyClaimSignatures<'info> {
    #[account(
        mut,
        seeds = [BRIDGE.as_bytes()], 
        bump
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + SignatureThreshold::INIT_SPACE,
        seeds = [hash(data.tx_hash.as_ref()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,

     /// CHECK: used to get instruction data
     #[account(address = Instructions::id())]
     pub instruction_acc: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VerifyClaimSignaturesData {
    tx_hash: [u8 ;32],
    claim_data: ClaimNftData,
    signatures: Vec<SignatureInfo>,
    validators_count: u64,
}

#[derive(Accounts)]
#[instruction(data: VerifyRewardValidatorSignaturesData)]
pub struct VerifyRewardValidatorSignatures<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + SignatureThreshold::INIT_SPACE,
        seeds = [hash([REWARD, &data.validator_public_key.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,

     /// CHECK: used to get instruction data
     #[account(address = Instructions::id())]
     pub instruction_acc: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VerifyRewardValidatorSignaturesData {
    validator_public_key: Pubkey,
    signatures: Vec<SignatureInfo>,
}

#[derive(Accounts)]
#[instruction(data: ValidatorClaimRewardData)]
pub struct ValidatorClaimReward<'info> {
    #[account(
        mut,
        seeds = [BRIDGE.as_bytes()], 
        bump
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(
        mut, 
        seeds = [hash([REWARD, &data.validator_public_key.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    #[account(
        mut,
        seeds = [hash([VALIDATOR, &data.validator_public_key.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub validators: Account<'info, Validators>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,

     /// CHECK: used to get instruction data
     #[account(address = Instructions::id())]
     pub instruction_acc: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ValidatorClaimRewardData {
    validator_public_key: Pubkey,
}

#[derive(Accounts)]
#[instruction(data: InitializeData)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed, 
        payer = user, 
        space = 8 + Bridge::INIT_SPACE, 
        seeds = [BRIDGE.as_bytes()], 
        bump
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + Validators::INIT_SPACE,
        seeds = [hash([VALIDATOR, &data.validator_public_key.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub validators: Account<'info, Validators>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeData {
    validator_public_key: Pubkey,
}

#[derive(Accounts)]
#[instruction(data: AddValidatorData)]
pub struct AddValidator<'info> {
    #[account(
        mut, 
        seeds = [BRIDGE.as_bytes()], 
        bump, 
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(
        mut, 
        seeds = [hash([THRESHOLD, &data.validator_public_key.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    #[account(
        init,
        payer = user,
        space = 8 + Validators::INIT_SPACE,
        seeds = [hash([VALIDATOR, &data.validator_public_key.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub validators: Account<'info, Validators>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: used to get instruction data
    #[account(address = Instructions::id())]
    pub instruction_acc: AccountInfo<'info>,
}
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddValidatorData {
    validator_public_key: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SignatureInfo {
    public_key: Pubkey,
    sig: [u8; 64],
}

#[derive(Accounts)]
#[instruction(data: LockData)]
pub struct Lock<'info> {
    #[account(
        mut, 
        seeds = [BRIDGE.as_bytes()], 
        bump = data.bridge_bump, 
    )]
    pub bridge: Account<'info, Bridge>,

    
    #[account(
        init_if_needed,
        payer = authority, 
        space = 8 + OtherTokenInfo::INIT_SPACE, 
        seeds = [hash([OTHER_TOKENS, &data.token_id.key().to_string(), SELF_CHAIN, &data.source_nft_contract_address.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub other_tokens: Account<'info, OtherTokenInfo>,

    #[account(
        init_if_needed,
        payer = authority, 
        space = 8 + SelfTokenInfo::INIT_SPACE,
        seeds = [hash([SELF_TOKENS, &data.token_id.key().to_string(), SELF_CHAIN, &data.source_nft_contract_address.key().to_string()].concat().as_bytes()).as_ref()], 
        bump
    )]
    pub self_tokens: Account<'info, SelfTokenInfo>,

    #[account(
        init_if_needed,
        payer = authority, 
        space = 8 + ContractInfo::INIT_SPACE, 
        seeds = [hash([DUPLICATE_TO_ORIGINAL_MAPPING, &data.source_nft_contract_address.key().to_string(), SELF_CHAIN].concat().as_bytes()).as_ref()],
        bump
    )]
    pub duplicate_to_original_mapping: Account<'info, ContractInfo>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = from.owner == authority.to_account_info().key() && from.mint == to.mint)]
    pub from: Account<'info, TokenAccount>,

    #[account(mut, constraint = to.owner == bridge.to_account_info().key())]
    pub to: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LockData {
    token_id: Pubkey,
    destination_chain: String,
    destination_user_address: String,
    source_nft_contract_address: Pubkey,
    token_amount: u64,
    bridge_bump: u8,
}

#[derive(Accounts)]
#[instruction(data: ClaimData)]
pub struct ClaimUnlock<'info> {
    #[account(
        mut, 
        seeds = [BRIDGE.as_bytes()], 
        bump, 
    )]
    pub bridge: Box<Account<'info, Bridge>>,

    #[account(
        mut,
        seeds = [hash(data.claim_data.transaction_hash.as_ref()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    // #[account(
    //     init_if_needed,
    //     payer = user, 
    //     space = 8 + OtherTokenInfo::INIT_SPACE, 
    //     seeds = [hash([OTHER_TOKENS,&data.nft_mint.unwrap().key().to_string(),SELF_CHAIN,&create_collection_mint.key().to_string()].concat().as_bytes()).as_ref()],
    //     bump
    // )]
    // pub other_tokens: Box<Account<'info, OtherTokenInfo>>,

    #[account(
        init_if_needed,
        payer = user, 
        space = 8 + SelfTokenInfo::INIT_SPACE,
        seeds = [hash([SELF_TOKENS, &data.claim_data.token_id, &data.claim_data.source_chain, &data.claim_data.source_nft_contract_address].concat().as_bytes()).as_ref()], 
        bump
    )]
    pub self_tokens: Box<Account<'info, SelfTokenInfo>>,

    #[account(
        init_if_needed,
        space = 8 + ContractInfo::INIT_SPACE,
        payer = user,
        seeds = [hash([ORIGINAL_TO_DUPLICATE_MAPPING, &data.claim_data.source_nft_contract_address, &data.claim_data.source_chain].concat().as_bytes()).as_ref()],
        bump
    )]
    pub original_to_duplicate_mapping: Box<Account<'info, ContractInfo>>,

    #[account(
        init_if_needed,
        space = 8 + ContractInfo::INIT_SPACE,
        payer = user,
        seeds = [hash([DUPLICATE_TO_ORIGINAL_MAPPING, &create_collection_mint.key().to_string(), SELF_CHAIN].concat().as_bytes()).as_ref()],
        bump
    )]
    pub duplicate_to_original_mapping: Box<Account<'info, ContractInfo>>,

    #[account(
        mut
    )]
    pub bridge_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [hash([data.claim_data.source_nft_contract_address.clone(), data.claim_data.source_chain.clone()].concat().as_bytes()).as_ref()],
        bump,
    )]
    pub create_collection_mint: Account<'info, Mint>,
    #[account(
        mut
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(data: ClaimData)]
pub struct ClaimCreateCollection<'info> {
    #[account(
        mut, 
        seeds = [BRIDGE.as_bytes()], 
        bump, 
    )]
    pub bridge: Box<Account<'info, Bridge>>,

    #[account(
        mut,
        seeds = [hash(data.claim_data.transaction_hash.as_ref()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    #[account(
        init_if_needed,
        payer = user, 
        space = 8 + OtherTokenInfo::INIT_SPACE, 
        seeds = [hash([OTHER_TOKENS,&nft_mint.key().to_string(),SELF_CHAIN,&create_collection_mint.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub other_tokens: Box<Account<'info, OtherTokenInfo>>,

    #[account(
        init_if_needed,
        payer = user, 
        space = 8 + SelfTokenInfo::INIT_SPACE,
        seeds = [hash([SELF_TOKENS, &data.claim_data.token_id, &data.claim_data.source_chain, &data.claim_data.source_nft_contract_address].concat().as_bytes()).as_ref()], 
        bump
    )]
    pub self_tokens: Box<Account<'info, SelfTokenInfo>>,

    #[account(
        init_if_needed,
        space = 8 + ContractInfo::INIT_SPACE,
        payer = user,
        seeds = [hash([ORIGINAL_TO_DUPLICATE_MAPPING, &data.claim_data.source_nft_contract_address, &data.claim_data.source_chain].concat().as_bytes()).as_ref()],
        bump
    )]
    pub original_to_duplicate_mapping: Box<Account<'info, ContractInfo>>,

    #[account(
        init_if_needed,
        space = 8 + ContractInfo::INIT_SPACE,
        payer = user,
        seeds = [hash([DUPLICATE_TO_ORIGINAL_MAPPING, &create_collection_mint.key().to_string(), SELF_CHAIN].concat().as_bytes()).as_ref()],
        bump
    )]
    pub duplicate_to_original_mapping: Box<Account<'info, ContractInfo>>,

    #[account(
        init_if_needed,
        seeds = [hash([data.claim_data.source_nft_contract_address.clone(), data.claim_data.source_chain.clone()].concat().as_bytes()).as_ref()],
        bump,
        payer = user,
        mint::decimals = 0,
        mint::authority = create_collection_mint,
        mint::freeze_authority = create_collection_mint
    )]
    pub create_collection_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
        mut,
        address=find_metadata_account(&create_collection_mint.key()).0
    )]
    pub create_collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=find_master_edition_account(&create_collection_mint.key()).0
    )]
    pub create_collection_master_edition: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = create_collection_mint,
        associated_token::authority = user
    )]
    pub create_collection_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        mint::decimals = 0,
        mint::authority = create_collection_mint,
        mint::freeze_authority = create_collection_mint
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
        mut,
        address=find_metadata_account(&nft_mint.key()).0
    )]
    pub nft_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=find_master_edition_account(&nft_mint.key()).0
    )]
    pub nft_master_edition: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(data: ClaimData)]
pub struct ClaimCreateNft<'info> {
    #[account(
        mut, 
        seeds = [BRIDGE.as_bytes()], 
        bump, 
    )]
    pub bridge: Box<Account<'info, Bridge>>,

    #[account(
        mut,
        seeds = [hash(data.claim_data.transaction_hash.as_ref()).as_ref()],
        bump
    )]
    pub threshold: Account<'info, SignatureThreshold>,

    #[account(
        init_if_needed,
        payer = user, 
        space = 8 + OtherTokenInfo::INIT_SPACE, 
        seeds = [hash([OTHER_TOKENS,&nft_mint.key().to_string(),SELF_CHAIN,&create_collection_mint.key().to_string()].concat().as_bytes()).as_ref()],
        bump
    )]
    pub other_tokens: Box<Account<'info, OtherTokenInfo>>,

    #[account(
        init_if_needed,
        payer = user, 
        space = 8 + SelfTokenInfo::INIT_SPACE,
        seeds = [hash([SELF_TOKENS, &data.claim_data.token_id, &data.claim_data.source_chain, &data.claim_data.source_nft_contract_address].concat().as_bytes()).as_ref()], 
        bump
    )]
    pub self_tokens: Box<Account<'info, SelfTokenInfo>>,

    #[account(
        init_if_needed,
        space = 8 + ContractInfo::INIT_SPACE,
        payer = user,
        seeds = [hash([ORIGINAL_TO_DUPLICATE_MAPPING, &data.claim_data.source_nft_contract_address, &data.claim_data.source_chain].concat().as_bytes()).as_ref()],
        bump
    )]
    pub original_to_duplicate_mapping: Box<Account<'info, ContractInfo>>,

    // #[account(
    //     init_if_needed,
    //     space = 8 + ContractInfo::INIT_SPACE,
    //     payer = user,
    //     seeds = [hash([DUPLICATE_TO_ORIGINAL_MAPPING, &create_collection_mint.key().to_string(), SELF_CHAIN].concat().as_bytes()).as_ref()],
    //     bump
    // )]
    // pub duplicate_to_original_mapping: Box<Account<'info, ContractInfo>>,

    #[account(
        mut,
        seeds = [hash([data.claim_data.source_nft_contract_address.clone(), data.claim_data.source_chain.clone()].concat().as_bytes()).as_ref()],
        bump,
    )]
    pub create_collection_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
        mut,
        address=find_metadata_account(&create_collection_mint.key()).0
    )]
    pub create_collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=find_master_edition_account(&create_collection_mint.key()).0
    )]
    pub create_collection_master_edition: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        mint::decimals = 0,
        mint::authority = create_collection_mint,
        mint::freeze_authority = create_collection_mint
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
        mut,
        address=find_metadata_account(&nft_mint.key()).0
    )]
    pub nft_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=find_master_edition_account(&nft_mint.key()).0
    )]
    pub nft_master_edition: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ClaimData {
    pub claim_data: ClaimNftData,
    pub nft_mint: Option<Pubkey>,
    // pub signatures: Vec<SignatureInfo>,
    // pub bridge_bump: u8,
    // pub other_tokens_bump: u8,
    // pub self_tokens_bump: u8,
    // pub collection_bump: u8,
    // pub collection_mint_key: Pubkey
}

 #[derive(AnchorSerialize, AnchorDeserialize)] 
 pub struct ClaimNftData {
    pub token_id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub destination_user_address: Pubkey,
    pub source_nft_contract_address: String,
    pub name: String,
    pub symbol: String,
    pub royalty: u64,
    pub royalty_receiver: Pubkey,
    pub metadata: String,
    pub transaction_hash: String,
    pub token_amount: u64,
    pub nft_type: String,
    pub fee: u64,
 }

//Events
#[event]
pub struct LockEvent {
    token_id: String,
    destination_chain: String,
    destination_user_address: String,
    source_nft_contract_address: String,
    token_amount: u64,
    nft_type: String,
    chain: String,
}

#[event]
pub struct ClaimEvent {
    source_chain: String,
    transaction_hash: String
}

#[event]
pub struct RewardValidatorEvent {
    validator_public_key: Pubkey,
}