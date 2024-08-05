use cosmwasm_std::Addr;
// use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use cw_storage_plus::{Item, Map};

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

pub static VALIDATORS_STORAGE: Map<Vec<u8>, Validator> = Map::new("v_s");
pub static BLACKLISTED_VALIDATORS: Map<Vec<u8>, bool> = Map::new("bvs");

pub static UNIQUE_IDENTIFIER_STORAGE: Map<[u8; 32], bool> = Map::new("ui_s");

pub static ORIGINAL_TO_DUPLICATE_STORAGE: Map<(String, String), OriginalToDuplicateContractInfo> =
    Map::new("otdm_s");

pub static DUPLICATE_TO_ORIGINAL_STORAGE: Map<(Addr, String), DuplicateToOriginalContractInfo> =
    Map::new("dtom_s");

pub static ORIGINAL_STORAGE_721: Map<(String, String), Addr> = Map::new("o721_s");

pub static DUPLICATE_STORAGE_721: Map<(String, String), Addr> = Map::new("d721_s");

pub static NFT_COLLECTION_OWNER: Map<(String, String), (Addr, u128)> = Map::new("nco");

pub const COLLETION_DEPLOYER_CODE: Item<u64> = Item::new("cd_v");

pub const STORAGE_DEPLOYER_CODE: Item<u64> = Item::new("sd_v");

pub const CONFIG: Item<State> = Item::new("config");
