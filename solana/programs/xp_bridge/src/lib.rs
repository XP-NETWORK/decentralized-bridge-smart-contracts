pub mod error;
pub mod state;

use anchor_lang::{prelude::*, solana_program::{sysvar::SysvarId, pubkey}};
use anchor_spl::{
    self,
    token::{self, Burn, Mint, Token, TokenAccount, Transfer}, associated_token::AssociatedToken, metadata::Metadata,
};
use mpl_token_metadata::{state::{Collection, Creator, UseMethod, Uses}, pda::{find_metadata_account, find_master_edition_account}};
use state::{Bridge, Validators, ContractInfo, OtherTokenInfo, SelfTokenInfo};
use std::mem::size_of;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::sysvar::instructions::{ID as IX_ID, load_instruction_at_checked};

use anchor_lang::solana_program::ed25519_program::ID as ED25519_ID;

use std::convert::TryInto;
declare_id!("6FtWDSrBY6XBip57XY1wq2hjhJooKQTVsPDNdziYq615");

#[constant]
    pub const SEED: &str = "Collection";

#[program]
pub mod xp_bridge {
    use anchor_lang::solana_program::{
        native_token::LAMPORTS_PER_SOL, program::invoke, system_instruction,
    };
    use mpl_token_metadata::assertions::collection;
    // use crate::utils::{transfer_service_fee_lamports};

    pub const SELF_CHAIN: &[u8] = b"SOL";
    pub const TYPE_ERC721: &[u8] = b"singular"; // a more general term to accomodate non-evm chains
    pub const TYPE_ERC1155: &[u8] = b"multiple"; // a more general term to accomodate non-evm chains

    use super::*;

    // Initializes a new bridge account with a set of validators and a threshold.
    pub fn initialize(ctx: Context<Initialize>, data: InitializeData) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.validator_count += 1;

        let validator_map = &mut ctx.accounts.validators;
        validator_map.added = true;
        validator_map.pending_rewards = 0;

        Ok(())
    }

    pub fn add_validator(ctx: Context<AddValidator>, data: AddValidatorData) -> Result<()> {

        if data.signatures.len() == 0 {
            return Err(error::ErrorCode::NoSignatures.into());
        }

        // let mut percentage: u64 = 0;

        // for arg in data.signatures.into_iter() {
        let ix: Instruction = load_instruction_at_checked(0, &ctx.accounts.instruction_acc)?;

            // Check that ix is what we expect to have been sent
        utils::verify_ed25519_ix(&ix, &data.signatures[0].public_key.to_bytes(), &data.public_key.to_bytes(), &data.signatures[0].sig)?;

        //     ed25519_program::

        //     let mut dest_slice = [0u8; 64];
        //     let _ = arg.sig.load_slice(0, &mut dest_slice);

        //     let res = self.verify_ed25519(
        //         &arg.public_key.to_byte_array(),
        //         &new_validator_public_key.to_byte_array(),
        //         &dest_slice,
        //     );
        //     if res {
        //         let lene = self.validators(&arg.public_key).is_empty();
        //         if lene == false && self.validators(&arg.public_key).get(1).added {
        //             percentage = percentage + 1;
        //         }
        //     }
        // }

        // let validators_count = self.validators_count().get();

        // require!(
        //     percentage >= (((validators_count * 2) / 3) + 1),
        //     "Threshold not reached!"
        // );


        let bridge = &mut ctx.accounts.bridge;
        bridge.validator_count += 1;

        let validator_map = &mut ctx.accounts.validators;
        validator_map.added = true;
        validator_map.pending_rewards = 0;
        Ok(())
    }

    pub fn lock_721(ctx: Context<Lock721>, data: Lock721Data)->Result<()>{
        let other_tokens =&mut ctx.accounts.other_tokens;
        let self_tokens =&mut ctx.accounts.self_tokens;

        let am = &&mut 0u64;
        let duplicate_to_original_mapping = &mut ctx.accounts.duplicate_to_original_mapping;
        
        let binding = duplicate_to_original_mapping.to_account_info();
        let binding1 = other_tokens.to_account_info();
        let binding2 = self_tokens.to_account_info();
        
        // let is_token_exists =   tokens_vec.iter().find(|&x| 
        //     x.self_token_id == data.token_id && 
        //     x.self_chain.as_bytes() == SELF_CHAIN && 
        //     x.self_contract_address == data.source_nft_contract_address);

        let is_token_exists;

        if binding1.try_borrow_lamports()?.gt(am)  {
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
            1,
        )?;
        if binding.try_borrow_lamports()?.gt(am) {
            emit!(Lock721Event { 
                token_id: mut_token_id, 
                destination_chain: data.destination_chain, 
                destination_user_address: data.destination_user_address, 
                source_nft_contract_address: ctx.accounts.duplicate_to_original_mapping.address.clone(), 
                token_amount: 1, 
                nft_type: "singular".to_string(), 
                chain: ctx.accounts.duplicate_to_original_mapping.chain.clone()});
        }
        else{
            emit!(Lock721Event { 
                token_id: mut_token_id, 
                destination_chain: data.destination_chain, 
                destination_user_address: data.destination_user_address, 
                source_nft_contract_address: data.source_nft_contract_address.to_string(), 
                token_amount: 1, 
                nft_type: "singular".to_string(), 
                chain: "SOL".to_string()});
        } 
        
        Ok(())
    }

    pub fn claim_721(ctx: Context<Claim721>, data: ClaimData721) -> Result<()> {

        if !data.data.nft_type.as_bytes().eq(TYPE_ERC721) {
            return Err(error::ErrorCode::InvalidNft.into());
        }

        let self_tokens =&mut ctx.accounts.self_tokens;

        let am = &&mut 0u64;
        let original_to_duplicate_mapping = &mut ctx.accounts.original_to_duplicate_mapping;
        let binding = original_to_duplicate_mapping.to_account_info();
        let binding1 = self_tokens.to_account_info();


        
        let is_token_exists;

            if binding1.try_borrow_lamports()?.gt(am)  {
                is_token_exists = Some(self_tokens);
            }
            else{
                is_token_exists = Option::None;
            }
    

        if binding.try_borrow_lamports()?.gt(am) {
            // isDuplicate
            match is_token_exists {
                Some(_v)=>{
                    // Unlock721
                    let transfer_ctx = Transfer {
                        from: ctx.accounts.bridge.to_account_info(),
                        to: ctx.accounts.to.to_account_info(),
                        authority: ctx.accounts.bridge.to_account_info(),
                    };
                    let auth_seeds = [b"bridge".as_ref(), &[data.bridge_bump]];

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
                    let _ = collection_createor::create_nft_in_collection(&ctx, data.data.metadata, data.data.name, data.data.symbol);

                    let transfer_ctx = Transfer {
                        from: ctx.accounts.create_collection_mint.to_account_info(),
                        to: ctx.accounts.to.to_account_info(),
                        authority: ctx.accounts.create_collection_mint.to_account_info(),
                    };
                    let auth_seeds = [b"collection".as_ref(), &[data.collection_bump]];

                    token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(), 
                            transfer_ctx,
                            &[&auth_seeds]
                        ),
                        1,
                    )?;
                }
            }
        }
        else{
            // notDuplicate
            match is_token_exists {
                Some(_v)=>{
                    // Unlock721
                    let transfer_ctx = Transfer {
                        from: ctx.accounts.bridge.to_account_info(),
                        to: ctx.accounts.to.to_account_info(),
                        authority: ctx.accounts.bridge.to_account_info(),
                    };
                    let auth_seeds = [b"bridge".as_ref(), &[data.bridge_bump]];

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
                    let _ = collection_createor::create_collection_nft(&ctx, data.data.metadata.clone(), data.data.name.clone(), data.data.symbol.clone());

                    let _ = collection_createor::create_nft_in_collection(&ctx, data.data.metadata, data.data.name, data.data.symbol);
                    
                    let otdm = &mut ctx.accounts.original_to_duplicate_mapping;
                    let dtom = &mut ctx.accounts.duplicate_to_original_mapping;


                    let other_tokens = &mut ctx.accounts.other_tokens;
                    let self_tokens = &mut ctx.accounts.self_tokens;

                    otdm.chain = "SOL".to_string();
                    otdm.address = ctx.accounts.create_collection_mint.key().to_string();

                    dtom.chain = data.data.source_chain.clone();
                    dtom.address = data.data.source_nft_contract_address.clone();

                    other_tokens.token_id = data.data.token_id;
                    other_tokens.chain = data.data.source_chain.clone();
                    other_tokens.contract_address = data.data.source_nft_contract_address.clone();

                    self_tokens.token_id = ctx.accounts.nft_token_account.key();
                    self_tokens.chain = "SOL".to_string();
                    self_tokens.contract_address = ctx.accounts.create_collection_mint.key().to_string();

                    let transfer_ctx = Transfer {
                        from: ctx.accounts.create_collection_mint.to_account_info(),
                        to: ctx.accounts.to.to_account_info(),
                        authority: ctx.accounts.create_collection_mint.to_account_info(),
                    };
                    
                    let auth_seeds = [b"collection".as_ref(), &[data.collection_bump]];

                    token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(), 
                            transfer_ctx,
                            &[&auth_seeds]
                        ),
                        1,
                    )?;

                }
            }
        }
        emit!(Claim721Event{
            source_chain: data.data.source_chain,
            transaction_hash: data.data.transaction_hash
        });
        Ok(())
    }
    

    // pub fn validate_pause(ctx: Context<ValidatePause>, data: PauseData) -> Result<()> {
    //     let bridge = &mut ctx.accounts.bridge;

    //     let consumed_action = &mut ctx.accounts.consumed_action;
    //     consumed_action.consumed = true;

    //     validate_action(&ctx.accounts.instruction_acc, bridge, data.try_to_vec()?)?;

    //     bridge.paused = true;
    //     Ok(())
    // }

    // pub fn validate_unpause(ctx: Context<ValidateUnpause>, data: UnpauseData) -> Result<()> {
    //     let bridge = &mut ctx.accounts.bridge;

    //     let consumed_action = &mut ctx.accounts.consumed_action;
    //     consumed_action.consumed = true;

    //     validate_action(&ctx.accounts.instruction_acc, bridge, data.try_to_vec()?)?;

    //     bridge.paused = false;
    //     Ok(())
    // }

    // pub fn validate_withdraw_fees(
    //     ctx: Context<WithdrawFees>,
    //     data: WithdrawFeesData,
    // ) -> Result<()> {
    //     let bridge = &ctx.accounts.bridge;

    //     let consumed_action = &mut ctx.accounts.consumed_action;
    //     consumed_action.consumed = true;

    //     validate_action(&ctx.accounts.instruction_acc, bridge, data.try_to_vec()?)?;

    //     let from_account = &ctx.accounts.bridge.to_account_info();
    //     let to_account = &ctx.accounts.user.to_account_info();

    //     let bridge_balance = **from_account.try_borrow_lamports()?;
    //     let min_balance = LAMPORTS_PER_SOL / 100;

    //     if bridge_balance <= min_balance {
    //         return err!(error::ErrorCode::InsufficientFundsForTransaction);
    //     }

    //     let amount = bridge_balance - min_balance; // current balance of bridge - minimum balance of bridge
    //     transfer_service_fee_lamports(from_account, to_account, amount)?;

    //     Ok(())
    // }

    // pub fn validate_update_groupkey(
    //     ctx: Context<ValidateUpdateGroupkey>,
    //     data: UpdateGroupkeyData,
    // ) -> Result<()> {
    //     let bridge = &mut ctx.accounts.bridge;

    //     let consumed_action = &mut ctx.accounts.consumed_action;
    //     consumed_action.consumed = true;

    //     validate_action(&ctx.accounts.instruction_acc, bridge, data.try_to_vec()?)?;

    //     bridge.group_key = data.new_key;
    //     Ok(())
    // }

    // // Transfer wrapped NFT to Solana account
    // pub fn validate_transfer_nft(
    //     ctx: Context<ValidateTransferNft>,
    //     data: TransferNftData,
    // ) -> Result<()> {
    //     let bridge = &mut ctx.accounts.bridge;

    //     let consumed_action = &mut ctx.accounts.consumed_action;
    //     consumed_action.consumed = true;

    //     validate_action(&ctx.accounts.instruction_acc, bridge, data.try_to_vec()?)?;

    //     let seller_fee_basis_points = match data.seller_fee_basis_points {
    //         Some(r) => r,
    //         None => 0,
    //     };

    //     let datav2 = AnchorDataV2 {
    //         name: data.name,
    //         symbol: data.symbol,
    //         uri: data.uri,
    //         seller_fee_basis_points,
    //         creators: data.creators,
    //         collection: data.collection,
    //         uses: None,
    //     };

    //     let mint_to_ctx = token::MintTo {
    //         mint: ctx.accounts.mint.to_account_info(),
    //         to: ctx.accounts.token_account.to_account_info(),
    //         authority: ctx.accounts.authority.to_account_info(),
    //     };

    //     let auth_seeds = ["auth".as_bytes(), &[data.auth_bump]];

    //     token::mint_to(
    //         CpiContext::new_with_signer(
    //             ctx.accounts.token_program.to_account_info(),
    //             mint_to_ctx,
    //             &[&auth_seeds],
    //         ),
    //         1,
    //     )?;

    //     create_metadata_accounts_v2(
    //         CpiContext::new_with_signer(
    //             ctx.accounts.metadata_program.to_account_info(),
    //             ctx.accounts.clone(),
    //             &[&auth_seeds],
    //         ),
    //         false,
    //         true,
    //         datav2.into(),
    //     )?;

    //     create_master_edition_v3(
    //         CpiContext::new_with_signer(
    //             ctx.accounts.metadata_program.to_account_info(),
    //             ctx.accounts.clone(),
    //             &[&auth_seeds],
    //         ),
    //         None,
    //     )?;

    //     Ok(())
    // }

    // pub fn withdraw_nft(
    //     ctx: Context<WithdrawNft>,
    //     chain_nonce: u8,
    //     to: String,
    //     lamports: u64,
    //     _bridge_bump: u8,
    // ) -> Result<()> {
    //     if lamports == 0 {
    //         return err!(error::ErrorCode::InsufficientFundsForTransaction);
    //     }
    //     // transfer service fee to bridge PDA
    //     invoke(
    //         &system_instruction::transfer(
    //             &ctx.accounts.authority.to_account_info().key(),
    //             &ctx.accounts.bridge.to_account_info().key(),
    //             lamports,
    //         ),
    //         &[
    //             ctx.accounts.authority.to_account_info().clone(),
    //             ctx.accounts.bridge.to_account_info().clone(),
    //             ctx.accounts.system_program.to_account_info().clone(),
    //         ],
    //     )?;

    //     let burn_ctx = Burn {
    //         mint: ctx.accounts.mint.to_account_info(),
    //         from: ctx.accounts.token_account.to_account_info(),
    //         authority: ctx.accounts.authority.to_account_info(),
    //     };

    //     token::burn(
    //         CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_ctx),
    //         1,
    //     )?;

    //     ctx.accounts.bridge.action_id += 1;

    //     emit!(UnfreezeNft {
    //         chain_nonce: chain_nonce,
    //         to: to,
    //         action_id: ctx.accounts.bridge.action_id,
    //         mint: ctx.accounts.mint.key(),
    //         lamports,
    //     });
    //     Ok(())
    // }

    // pub fn freeze_nft(
    //     ctx: Context<FreezeNft>,
    //     chain_nonce: u8,
    //     to: String,
    //     lamports: u64,
    //     mint_with: String,
    //     _bridge_bump: u8,
    // ) -> Result<()> {
    //     if lamports == 0 {
    //         return err!(error::ErrorCode::InsufficientFundsForTransaction);
    //     }
    //     // transfer service fee to bridge PDA
    //     invoke(
    //         &system_instruction::transfer(
    //             &ctx.accounts.authority.to_account_info().key(),
    //             &ctx.accounts.bridge.to_account_info().key(),
    //             lamports,
    //         ),
    //         &[
    //             ctx.accounts.authority.to_account_info().clone(),
    //             ctx.accounts.bridge.to_account_info().clone(),
    //             ctx.accounts.system_program.to_account_info().clone(),
    //         ],
    //     )?;

    //     let transfer_ctx = Transfer {
    //         from: ctx.accounts.from.to_account_info(),
    //         to: ctx.accounts.to.to_account_info(),
    //         authority: ctx.accounts.authority.to_account_info(),
    //     };

    //     token::transfer(
    //         CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_ctx),
    //         1,
    //     )?;

    //     ctx.accounts.bridge.action_id += 1;

    //     emit!(TransferNft {
    //         chain_nonce: chain_nonce,
    //         to: to,
    //         mint: ctx.accounts.to.mint,
    //         action_id: ctx.accounts.bridge.action_id,
    //         mint_with,
    //         lamports
    //     });

    //     Ok(())
    // }

    // pub fn validate_unfreeze_nft(
    //     ctx: Context<ValidateUnfreezeNft>,
    //     data: UnfreezeNftData,
    // ) -> Result<()> {
    //     let bridge = &mut ctx.accounts.bridge;

    //     let consumed_action = &mut ctx.accounts.consumed_action;
    //     consumed_action.consumed = true;

    //     validate_action(&ctx.accounts.instruction_acc, bridge, data.try_to_vec()?)?;

    //     let transfer_ctx = Transfer {
    //         from: ctx.accounts.from.to_account_info(),
    //         to: ctx.accounts.to.to_account_info(),
    //         authority: ctx.accounts.bridge.to_account_info(),
    //     };

    //     let auth_seeds = [b"bridge".as_ref(), &[data.bridge_bump]];

    //     token::transfer(
    //         CpiContext::new_with_signer(
    //             ctx.accounts.token_program.to_account_info(),
    //             transfer_ctx,
    //             &[&auth_seeds],
    //         ),
    //         1,
    //     )?;
    //     Ok(())
    // }
}


pub mod utils {
    use anchor_lang::solana_program::{self};

    use super::*;
    /// Verify Ed25519Program instruction fields
    pub fn verify_ed25519_ix(ix: &Instruction, pubkey: &[u8], msg: &[u8], sig: &[u8]) -> Result<()> {
        if  ix.program_id       != ED25519_ID                   ||  // The program id we expect
            ix.accounts.len()   != 0                            ||  // With no context accounts
            ix.data.len()       != (16 + 64 + 32 + msg.len())       // And data of this size
        {
            return Err(error::ErrorCode::InvalidSignature.into());    // Otherwise, we can already throw err
        }

        check_ed25519_data(&ix.data, pubkey, msg, sig)?;            // If that's not the case, check data

        Ok(())
    }

    /// Verify serialized Ed25519Program instruction data
    pub fn check_ed25519_data(data: &[u8], pubkey: &[u8], msg: &[u8], sig: &[u8]) -> Result<()> {

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
        let message_data_size               = &data[12..=13];    // Bytes 12,13
        let message_instruction_index       = &data[14..=15];    // Bytes 14,15

        let data_pubkey                     = &data[16..16+32];  // Bytes 16..16+32
        let data_sig                        = &data[48..48+64];  // Bytes 48..48+64
        let data_msg                        = &data[112..];      // Bytes 112..end

        // Expected values

        let exp_public_key_offset:      u16 = 16; // 2*u8 + 7*u16
        let exp_signature_offset:       u16 = exp_public_key_offset + pubkey.len() as u16;
        let exp_message_data_offset:    u16 = exp_signature_offset + sig.len() as u16;
        let exp_num_signatures:          u8 = 1;
        let exp_message_data_size:      u16 = msg.len().try_into().unwrap();

        // Header and Arg Checks

        // Header
        if  num_signatures                  != &exp_num_signatures.to_le_bytes()        ||
            padding                         != &[0]                                     ||
            signature_offset                != &exp_signature_offset.to_le_bytes()      ||
            signature_instruction_index     != &u16::MAX.to_le_bytes()                  ||
            public_key_offset               != &exp_public_key_offset.to_le_bytes()     ||
            public_key_instruction_index    != &u16::MAX.to_le_bytes()                  ||
            message_data_offset             != &exp_message_data_offset.to_le_bytes()   ||
            message_data_size               != &exp_message_data_size.to_le_bytes()     ||
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

        Ok(())
    }
}


pub mod collection_createor{
    use anchor_lang::prelude::*;
    use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_sized_collection_item, sign_metadata, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata, SetAndVerifySizedCollectionItem, SignMetadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
    };
    use mpl_token_metadata::{
    pda::{find_master_edition_account, find_metadata_account},
    state::{CollectionDetails, Creator, DataV2},
    };

    use super::*;
    
    pub fn create_collection_nft(
        ctx: &Context<Claim721>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        // PDA for signing
        let signer_seeds: &[&[&[u8]]] = &[&[
            SEED.as_bytes(),
            &[*ctx.bumps.get("collection_mint").unwrap()],
        ]];

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
                    metadata: ctx.accounts.collection_metadata_account.to_account_info(),
                    mint: ctx.accounts.create_collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(), // use pda mint address as mint authority
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(), // use pda mint as update authority
                    payer: ctx.accounts.authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: name,
                symbol: symbol,
                uri: uri,
                seller_fee_basis_points: 0,
                creators: Some(vec![Creator {
                    address: ctx.accounts.authority.key(),
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
                    payer: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.create_collection_mint.to_account_info(),
                    edition: ctx.accounts.collection_master_edition.to_account_info(),
                    mint_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.create_collection_mint.to_account_info(),
                    metadata: ctx.accounts.collection_metadata_account.to_account_info(),
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
                creator: ctx.accounts.authority.to_account_info(),
                metadata: ctx.accounts.collection_metadata_account.to_account_info(),
            },
        ))?;

        Ok(())
    }

    pub fn create_nft_in_collection(
        ctx: &Context<Claim721>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            SEED.as_bytes(),
            &[*ctx.bumps.get("collection_mint").unwrap()],
        ]];

        // mint nft in collection
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.nft_token_account.to_account_info(),
                    authority: ctx.accounts.nft_collection_mint.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        // create metadata account for nft in collection
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.nft_metadata_account.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    mint_authority: ctx.accounts.nft_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.nft_collection_mint.to_account_info(),
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
                    mint_authority: ctx.accounts.nft_collection_mint.to_account_info(),
                    update_authority: ctx.accounts.nft_collection_mint.to_account_info(),
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
                    collection_authority: ctx.accounts.nft_collection_mint.to_account_info(),
                    payer: ctx.accounts.user.to_account_info(),
                    update_authority: ctx.accounts.nft_collection_mint.to_account_info(),
                    collection_mint: ctx.accounts.nft_collection_mint.to_account_info(),
                    collection_metadata: ctx.accounts.collection_metadata_account.to_account_info(),
                    collection_master_edition: ctx
                        .accounts
                        .collection_master_edition
                        .to_account_info(),
                },
                &signer_seeds,
            ),
            None,
        )?;

        Ok(())
    }

    #[derive(Accounts)]
    pub struct CreateCollectionNft<'info> {
        #[account(mut)]
        pub authority: Signer<'info>,
    
        #[account(
            init_if_needed,
            seeds = [SEED.as_bytes()],
            bump,
            payer = authority,
            mint::decimals = 0,
            mint::authority = collection_mint,
            mint::freeze_authority = collection_mint
        )]
        pub collection_mint: Account<'info, Mint>,
    
        /// CHECK:
        #[account(
            mut,
            address=find_metadata_account(&collection_mint.key()).0
        )]
        pub metadata_account: UncheckedAccount<'info>,
    
        /// CHECK:
        #[account(
            mut,
            address=find_master_edition_account(&collection_mint.key()).0
        )]
        pub master_edition: UncheckedAccount<'info>,
    
        #[account(
            init_if_needed,
            payer = authority,
            associated_token::mint = collection_mint,
            associated_token::authority = authority
        )]
        pub token_account: Account<'info, TokenAccount>,
    
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
        pub associated_token_program: Program<'info, AssociatedToken>,
        pub token_metadata_program: Program<'info, Metadata>,
        pub rent: Sysvar<'info, Rent>,
    }
    
    #[derive(Accounts)]
    pub struct CreateNftInCollection<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
    
        #[account(
            mut,
            seeds = [SEED.as_bytes()],
            bump,
        )]
        pub collection_mint: Account<'info, Mint>,
    
        /// CHECK:
        #[account(
            mut,
            address=find_metadata_account(&collection_mint.key()).0
        )]
        pub collection_metadata_account: UncheckedAccount<'info>,
    
        /// CHECK:
        #[account(
            mut,
            address=find_master_edition_account(&collection_mint.key()).0
        )]
        pub collection_master_edition: UncheckedAccount<'info>,
    
        #[account(
            init,
            payer = user,
            mint::decimals = 0,
            mint::authority = collection_mint,
            mint::freeze_authority = collection_mint
        )]
        pub nft_mint: Account<'info, Mint>,
    
        /// CHECK:
        #[account(
            mut,
            address=find_metadata_account(&nft_mint.key()).0
        )]
        pub metadata_account: UncheckedAccount<'info>,
    
        /// CHECK:
        #[account(
            mut,
            address=find_master_edition_account(&nft_mint.key()).0
        )]
        pub master_edition: UncheckedAccount<'info>,
    
        #[account(
            init_if_needed,
            payer = user,
            associated_token::mint = nft_mint,
            associated_token::authority = user
        )]
        pub token_account: Account<'info, TokenAccount>,
    
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
        pub associated_token_program: Program<'info, AssociatedToken>,
        pub token_metadata_program: Program<'info, Metadata>,
        pub rent: Sysvar<'info, Rent>,
    }
    
}
// #[derive(Accounts)]
// // #[instruction(data: InitializeData)]
// pub struct Initialize<'info> {
//     #[account(
//         init, 
//         payer = user, 
//         space = 8, 
//         seeds = ["bridge".as_bytes()], 
//         bump
//     )]
//     pub bridge: Account<'info, Bridge>,
//     // #[account(
//     //     init,
//     //     payer = user,
//     //     space = 8 + size_of::<Validators>(),
//     //     // seeds = [
//     //     //     data.public_key.key().as_ref(),
//     //     // ],
//     //     seeds = ["validator".as_bytes()],
//     //     bump
//     // )]
//     // pub validators_mapping: Account<'info, Validators>,
//     #[account(mut)]
//     pub user: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }
#[derive(Accounts)]
#[instruction(data: InitializeData)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed, 
        payer = user, 
        space = 8 + Bridge::INIT_SPACE, 
        seeds = [b"bridge".as_ref()], 
        bump
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + Validators::INIT_SPACE,
        seeds = [data.public_key.to_bytes().as_ref()],
        bump
    )]
    pub validators: Account<'info, Validators>,

    // #[account(
    //     init,
    //     payer = user,
    //     space = 8 + BridgeTokens::INIT_SPACE,
    //     seeds = [b"bridge_tokens".as_ref()],
    //     bump
    // )]
    // pub tokens: Account<'info, BridgeTokens>,

    // #[account(
    //     init_if_needed,
    //     payer = user,
    //     space = 8 + ContractInfo::INIT_SPACE,
    //     seeds = [b"otdm".as_ref()],
    //     bump
    // )]
    // pub original_to_duplicate_mapping: Account<'info, ContractInfo>,

    // #[account(
    //     init_if_needed,
    //     payer = user,
    //     space = 8 + ContractInfo::INIT_SPACE,
    //     seeds = [b"dtom".as_ref()],
    //     bump
    // )]
    // pub duplicate_to_original_mapping: Account<'info, ContractInfo>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeData {
    public_key: Pubkey,
}

#[derive(Accounts)]
#[instruction(data: AddValidatorData)]
pub struct AddValidator<'info> {
    #[account(
        mut, 
        seeds = [b"bridge".as_ref()], 
        bump, 
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(
        init,
        payer = user,
        space = 8 + Validators::INIT_SPACE,
        seeds = [data.public_key.to_bytes().as_ref()],
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
    public_key: Pubkey,
    signatures: Vec<SignatureInfo>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SignatureInfo {
    public_key: Pubkey,
    sig: [u8; 64],
}


#[derive(Accounts)]
#[instruction(data: Lock721Data)]
pub struct Lock721<'info> {
    #[account(
        mut, 
        seeds = [b"bridge".as_ref()], 
        bump = data.bridge_bump, 
    )]
    pub bridge: Account<'info, Bridge>,


    #[account(
        init_if_needed,
        payer = authority, 
        space = 8 + OtherTokenInfo::INIT_SPACE, 
        seeds = [b"other_tokens".as_ref(), data.token_id.to_bytes().as_ref(), b"SOL".as_ref(), data.source_nft_contract_address.to_bytes().as_ref()],
        bump
    )]
    pub other_tokens: Account<'info, OtherTokenInfo>,

    #[account(
        init_if_needed,
        payer = authority, 
        space = 8 + SelfTokenInfo::INIT_SPACE,
        seeds = [b"self_tokens".as_ref(), data.token_id.to_bytes().as_ref(), b"SOL".as_ref(), data.source_nft_contract_address.to_bytes().as_ref()], 
        bump
    )]
    pub self_tokens: Account<'info, SelfTokenInfo>,

    #[account(
        mut,
        seeds = [b"dtom".as_ref(), data.source_nft_contract_address.as_ref(), SELF_CHAIN],
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
pub struct Lock721Data {
    token_id: Pubkey,
    destination_chain: String,
    destination_user_address: String,
    source_nft_contract_address: Pubkey,
    bridge_bump: u8,
    other_tokens_bump: u8,
    self_tokens_bump: u8
}


#[derive(Accounts)]
#[instruction(data: ClaimData721)]
pub struct Claim721<'info> {
    #[account(
        mut, 
        seeds = [b"bridge".as_ref()], 
        bump, 
    )]
    pub bridge: Account<'info, Bridge>,

    #[account(
        init_if_needed,
        payer = authority, 
        space = 8 + OtherTokenInfo::INIT_SPACE, 
        seeds = [b"other_tokens".as_ref(), nft_collection_mint.key().as_ref(), b"SOL".as_ref(), create_collection_mint.key().as_ref()],
        bump
    )]
    pub other_tokens: Account<'info, OtherTokenInfo>,

    #[account(
        init_if_needed,
        payer = authority, 
        space = 8 + SelfTokenInfo::INIT_SPACE,
        seeds = [b"self_tokens".as_ref(), data.data.token_id.as_bytes(), b"SOL".as_ref(), data.data.source_nft_contract_address.as_bytes()], 
        bump
    )]
    pub self_tokens: Account<'info, SelfTokenInfo>,

    #[account(
        init_if_needed,
        space = 8 + ContractInfo::INIT_SPACE,
        payer = authority,
        seeds = [b"otdm".as_ref(), data.data.source_nft_contract_address.as_bytes().as_ref(), data.data.source_chain.as_bytes().as_ref()],
        bump
    )]
    pub original_to_duplicate_mapping: Account<'info, ContractInfo>,

    #[account(
        init_if_needed,
        space = 8 + ContractInfo::INIT_SPACE,
        payer = authority,
        seeds = [b"dtom".as_ref(), data.collection_mint_key.as_ref(), SELF_CHAIN.as_ref()],
        bump
    )]
    pub duplicate_to_original_mapping: Account<'info, ContractInfo>,

    #[account(mut, constraint = to.owner == data.data.destination_user_address)]
    pub to: Account<'info, TokenAccount>,

   
    /// CHECK: used to get instruction data
    #[account(address = Instructions::id())]
    pub instruction_acc: AccountInfo<'info>,



    //FOR CREATE COLLECTION ACCOUNTS
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init_if_needed,
        seeds = [SEED.as_ref()],
        bump,
        payer = authority,
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
        payer = authority,
        associated_token::mint = create_collection_mint,
        associated_token::authority = authority
    )]
    pub create_collection_token_account: Account<'info, TokenAccount>,
        
        

    //FOR CREATE NFT IN COLLECTION
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED.as_bytes()],
        bump,
    )]
    pub nft_collection_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
        mut,
        address=find_metadata_account(&nft_collection_mint.key()).0
    )]
    pub collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=find_master_edition_account(&nft_collection_mint.key()).0
    )]
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
        init,
        payer = user,
        mint::decimals = 0,
        mint::authority = nft_collection_mint,
        mint::freeze_authority = nft_collection_mint
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
}


#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ClaimData721 {
    pub data: ClaimData,
    pub signatures: Vec<SignatureInfo>,
    pub bridge_bump: u8,
    pub other_tokens_bump: u8,
    pub self_tokens_bump: u8,
    pub collection_bump: u8,
    pub collection_mint_key: Pubkey
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ClaimData {
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
pub struct Lock721Event {
    token_id: String,
    destination_chain: String,
    destination_user_address: String,
    source_nft_contract_address: String,
    token_amount: u8,
    nft_type: String,
    chain: String,
}

#[event]
pub struct Claim721Event {
    source_chain: String,
    transaction_hash: String
}
