use cosmwasm_std::Addr;

use secret_toolkit::storage::Item;

use crate::structs::CodeInfo;

pub const BLOCK_SIZE: usize = 256;

pub const SNIP721_INSTANTIATE_REPLY_ID: u64 = 5;

pub const SNIP1155_INSTANTIATE_REPLY_ID: u64 = 6;

pub const OWNER: Item<Addr> = Item::new(b"admin");

pub const SNIP721_CODE: Item<CodeInfo> = Item::new(b"s721_v");

pub const SNIP1155_CODE: Item<CodeInfo> = Item::new(b"s1155_v");
