
use common::CodeInfo;
use secret_toolkit::{
    serialization::Bincode2,
    storage::{Item, Keymap, KeymapBuilder, WithoutIter},
};

use cosmwasm_std::{Addr, Binary, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

use crate::structs::{
     DuplicateToOriginalContractInfo, OriginalToDuplicateContractInfo, State, Validator,
};

pub static CONFIG_KEY: &[u8] = b"config";

pub const BLOCK_SIZE: usize = 256;

pub const STORAGE_DEPLOYER_721_REPLY_ID: u64 = 1;
pub const STORAGE_DEPLOYER_1155_REPLY_ID: u64 = 2;

pub const STORAGE_DEPLOYER_REPLY_ID: u64 = 11;
pub const COLLECTION_DEPLOYER_REPLY_ID: u64 = 12;

pub const COLLECTION_DEPLOYER_721_REPLY_ID: u64 = 3;
pub const COLLECTION_DEPLOYER_1155_REPLY_ID: u64 = 4;

pub static VALIDATORS_STORAGE: Keymap<Binary, Validator, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"v_s").without_iter().build();

pub static BLACKLISTED_VALIDATORS: Keymap<Binary, bool, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"bv_s").without_iter().build();

pub static UNIQUE_IDENTIFIER_STORAGE: Keymap<[u8; 32], bool, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"ui_s").without_iter().build();

pub static ORIGINAL_TO_DUPLICATE_STORAGE: Keymap<
    (String, String),
    OriginalToDuplicateContractInfo,
    Bincode2,
    WithoutIter,
> = KeymapBuilder::new(b"otdm_s").without_iter().build();

pub static DUPLICATE_TO_ORIGINAL_STORAGE: Keymap<
    (Addr, String),
    DuplicateToOriginalContractInfo,
    Bincode2,
    WithoutIter,
> = KeymapBuilder::new(b"dtom_s").without_iter().build();

pub static ORIGINAL_STORAGE_721: Keymap<(String, String), (Addr, String), Bincode2, WithoutIter> =
    KeymapBuilder::new(b"o721_s").without_iter().build();

pub static ORIGINAL_STORAGE_1155: Keymap<(String, String), (Addr, String), Bincode2, WithoutIter> =
    KeymapBuilder::new(b"o1155_s").without_iter().build();

pub static DUPLICATE_STORAGE_721: Keymap<(String, String), (Addr, String), Bincode2, WithoutIter> =
    KeymapBuilder::new(b"d721_s").without_iter().build();

pub static DUPLICATE_STORAGE_1155: Keymap<(String, String), (Addr, String), Bincode2, WithoutIter> =
    KeymapBuilder::new(b"d1155_s").without_iter().build();

pub static CODEHASHES: Keymap<Addr, String, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"ch").without_iter().build();

pub static NFT_COLLECTION_OWNER: Keymap<(String, String), (Addr, u128), Bincode2, WithoutIter> =
    KeymapBuilder::new(b"nco").without_iter().build();

pub const COLLETION_DEPLOYER_CODE: Item<CodeInfo> = Item::new(b"cd_v");

pub const STORAGE_DEPLOYER_CODE: Item<CodeInfo> = Item::new(b"sd_v");

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}
