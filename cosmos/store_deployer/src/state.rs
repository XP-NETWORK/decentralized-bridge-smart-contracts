use cosmwasm_std::Addr;

use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("owner");

pub const STORAGE721_CODE: Item<u64> = Item::new("st721_v");

pub const STORAGE721_INSTANTIATE_REPLY_ID: u64 = 7;
