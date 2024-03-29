use cosmwasm_std::Addr;

use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("owner");

pub const COLLECTION721_ADDRESS: Item<Addr> = Item::new("c721_a");
