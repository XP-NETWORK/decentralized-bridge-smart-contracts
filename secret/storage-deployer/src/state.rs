use cosmwasm_std::Addr;

use secret_toolkit::storage::Item;

use crate::structs::CodeInfo;


pub const BLOCK_SIZE: usize = 256;

pub const STORAGE721_INSTANTIATE_REPLY_ID: u64 = 7;

pub const STORAGE1155_INSTANTIATE_REPLY_ID: u64 = 8;

pub const OWNER: Item<Addr> = Item::new(b"admin");

pub const STORAGE721_CODE: Item<CodeInfo> = Item::new(b"st721_v");

pub const STORAGE1155_CODE: Item<CodeInfo> = Item::new(b"st1155_v");
