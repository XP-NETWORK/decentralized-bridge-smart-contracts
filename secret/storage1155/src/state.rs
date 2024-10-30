use cosmwasm_std::Addr;

use secret_toolkit::storage::Item;

use crate::structs::CodeInfo;

pub const BLOCK_SIZE: usize = 256;

pub const OWNER: Item<Addr> = Item::new(b"owner");

pub const COLLECTION1155_CODE: Item<CodeInfo> = Item::new(b"c1155_c");

pub const COLLECTION1155_ADDRESS: Item<Addr> = Item::new(b"c1155_a");
