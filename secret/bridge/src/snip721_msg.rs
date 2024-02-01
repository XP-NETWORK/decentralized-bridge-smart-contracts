use schemars::JsonSchema;
use secret_toolkit::{
    snip721::{Expiration, Metadata},
    utils::HandleCallback,
};
use serde::{Deserialize, Serialize};

use crate::{state::BLOCK_SIZE, structs::RoyaltyInfo};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct SerialNumber {
    /// optional number of the mint run this token will be minted in.  A mint run represents a
    /// batch of NFTs released at the same time.  So if a creator decided to make 100 copies
    /// of an NFT, they would all be part of mint run number 1.  If they sold quickly, and
    /// the creator wanted to rerelease that NFT, he could make 100 more copies which would all
    /// be part of mint run number 2.
    pub mint_run: Option<u32>,
    /// serial number (in this mint run).  This is used to serialize
    /// identical NFTs
    pub serial_number: u32,
    /// optional total number of NFTs minted on this run.  This is used to
    /// represent that this token is number m of n
    pub quantity_minted_this_run: Option<u32>,
}

/// Factory handle messages to be used by offspring.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip721ExecuteMsg {
    MintNft {
        /// optional token id. if omitted, use current token index
        token_id: Option<String>,
        /// optional owner address. if omitted, owned by the message sender
        owner: Option<String>,
        /// optional public metadata that can be seen by everyone
        public_metadata: Option<Metadata>,
        /// optional private metadata that can only be seen by the owner and whitelist
        private_metadata: Option<Metadata>,
        /// optional serial number for this token
        serial_number: Option<SerialNumber>,
        /// optional royalty information for this token.  This will be ignored if the token is
        /// non-transferable
        royalty_info: Option<RoyaltyInfo>,
        /// optionally true if the token is transferable.  Defaults to true if omitted
        transferable: Option<bool>,
        /// optional memo for the tx
        memo: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    Approve {
        /// address being granted the permission
        spender: String,
        /// id of the token that the spender can transfer
        token_id: String,
        /// optional expiration for this approval
        expires: Option<Expiration>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// transfer a token if it is transferable
    TransferNft {
        /// recipient of the transfer
        recipient: String,
        /// id of the token to transfer
        token_id: String,
        /// optional memo for the tx
        memo: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
}

impl HandleCallback for Snip721ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum Collection721QueryMsg {
//     /// display the owner of the specified token if authorized to view it.  If the requester
//     /// is also the token's owner, the response will also include a list of any addresses
//     /// that can transfer this token.  The transfer approval list is for CW721 compliance,
//     /// but the NftDossier query will be more complete by showing viewing approvals as well
//     OwnerOf {
//         token_id: String,
//         /// optional address and key requesting to view the token owner
//         viewer: Option<ViewerInfo>,
//         /// optionally include expired Approvals in the response list.  If ommitted or
//         /// false, expired Approvals will be filtered out of the response
//         include_expired: Option<bool>,
//     },
// }

// impl Query for Collection721QueryMsg {
//     const BLOCK_SIZE: usize = 256;
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct Collection721OwnerOfResponse {
//     pub owner: Addr,
//     pub approvals: Vec<Cw721Approval>,
// }
