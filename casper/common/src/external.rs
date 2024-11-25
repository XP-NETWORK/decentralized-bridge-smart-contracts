#![allow(dead_code)]
pub mod collection {
    use alloc::string::{String, ToString};
    use casper_contract::contract_api::runtime;
    use casper_types::bytesrepr::{FromBytes, ToBytes};
    use casper_types::{runtime_args, CLType, CLTyped, ContractHash, Key, RuntimeArgs, URef};

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub enum TokenIdentifier {
        Index(u64),
        Hash(String),
    }
    impl CLTyped for TokenIdentifier {
        fn cl_type() -> CLType {
            CLType::String
        }
    }

    impl FromBytes for TokenIdentifier {
        fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
            let (tid, remainder) = String::from_bytes(bytes)?;
            match tid.parse::<u64>() {
                Ok(e) => Ok((TokenIdentifier::Index(e), remainder)),
                Err(_) => Ok((TokenIdentifier::Hash(tid), remainder)),
            }
        }
    }
    impl ToBytes for TokenIdentifier {
        fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, casper_types::bytesrepr::Error> {
            match self {
                TokenIdentifier::Index(e) => e.to_string().to_bytes(),
                TokenIdentifier::Hash(hash) => hash.to_bytes(),
            }
        }

        fn serialized_length(&self) -> usize {
            match self {
                TokenIdentifier::Index(e) => e.to_string().serialized_length(),
                TokenIdentifier::Hash(h) => h.serialized_length(),
            }
        }
    }

    const ENTRY_POINT_MINT: &str = "mint";
    const ARG_TOKEN_OWNER: &str = "token_owner";
    const ARG_TOKEN_META_DATA: &str = "token_meta_data";
    const ARG_TOKEN_ID: &str = "token_id";
    const ARG_TOKEN_HASH: &str = "token_hash";
    const ENTRY_POINT_BURN: &str = "burn";
    const ARG_TARGET_KEY: &str = "target_key";
    const ARG_SOURCE_KEY: &str = "source_key";
    const ENTRY_POINT_TRANSFER: &str = "transfer";
    const ENTRY_POINT_METADATA: &str = "metadata";
    pub const ENTRY_POINT_REGISTER_OWNER: &str = "register_owner";
    const ENTRY_POINT_OWNER_OF: &str = "owner_of";

    pub fn mint(nft_contract: ContractHash, token_owner: Key, token_metadata: String) {
        let (_, _, _token_id_string) = runtime::call_contract::<(String, Key, String)>(
            nft_contract,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_OWNER => token_owner,
                ARG_TOKEN_META_DATA => token_metadata,
            },
        );
    }

    // pub fn _metadata(nft_contract: ContractHash, tid: TokenIdentifier) -> String {
    //     let (meta,) = match tid {
    //         TokenIdentifier::Index(token_idx) => runtime::call_contract::<(String,)>(
    //             nft_contract,
    //             ENTRY_POINT_METADATA,
    //             runtime_args! {
    //             ARG_TOKEN_ID => token_idx,
    //                             },
    //         ),
    //         TokenIdentifier::Hash(token_hash) => runtime::call_contract::<(String,)>(
    //             nft_contract,
    //             ENTRY_POINT_METADATA,
    //             runtime_args! {
    //             ARG_TOKEN_HASH => token_hash,
    //                             },
    //         ),
    //     };
    //     meta
    // }

    pub fn owner_of(nft_contract: ContractHash, tid: TokenIdentifier) -> Key {
        let key = match tid {
            TokenIdentifier::Index(idx) => runtime::call_contract::<Key>(
                nft_contract,
                ENTRY_POINT_OWNER_OF,
                runtime_args! {
                    ARG_TOKEN_ID => idx,
                },
            ),
            TokenIdentifier::Hash(token_hash) => runtime::call_contract::<Key>(
                nft_contract,
                ENTRY_POINT_OWNER_OF,
                runtime_args! {
                    ARG_TOKEN_HASH => token_hash,
                },
            ),
        };
        key
    }

    pub fn transfer(
        nft_contract: ContractHash,
        source_key: Key,
        target_key: Key,
        tid: TokenIdentifier,
    ) {
        let (_, _) = match tid {
            TokenIdentifier::Index(idx) => runtime::call_contract::<(String, Key)>(
                nft_contract,
                ENTRY_POINT_TRANSFER,
                runtime_args! {
                    ARG_TOKEN_ID => idx,
                    ARG_TARGET_KEY => target_key,
                    ARG_SOURCE_KEY => source_key
                },
            ),
            TokenIdentifier::Hash(token_hash) => runtime::call_contract::<(String, Key)>(
                nft_contract,
                ENTRY_POINT_TRANSFER,
                runtime_args! {
                    ARG_TOKEN_HASH => token_hash,
                    ARG_TARGET_KEY => target_key,
                    ARG_SOURCE_KEY => source_key
                },
            ),
        };
    }

    pub fn register(nft_contract: ContractHash, target_key: Key) {
        let (_collection_name, _) = runtime::call_contract::<(String, URef)>(
            nft_contract,
            ENTRY_POINT_REGISTER_OWNER,
            runtime_args! {
                ARG_TOKEN_OWNER => target_key,
            },
        );
    }
}

pub mod storage {
    use crate::collection::TokenIdentifier;
    use casper_contract::contract_api::runtime;
    use casper_types::account::AccountHash;
    use casper_types::{runtime_args, ContractHash, RuntimeArgs};

    const ENTRY_POINT_STORAGE_UNLOCK_TOKEN: &str = "unlock_token";
    const ARG_TOKEN_ID: &str = "token_id";
    const ARG_TO: &str = "to_arg";

    pub fn unlock_token(
        storage_contract: ContractHash,
        token_id: TokenIdentifier,
        to: AccountHash,
    ) {
        match token_id {
            TokenIdentifier::Index(token_idx) => runtime::call_contract::<()>(
                storage_contract,
                ENTRY_POINT_STORAGE_UNLOCK_TOKEN,
                runtime_args! {
                    ARG_TOKEN_ID => token_idx,
                    ARG_TO => to
                },
            ),
            TokenIdentifier::Hash(token_hash) => runtime::call_contract::<()>(
                storage_contract,
                ENTRY_POINT_STORAGE_UNLOCK_TOKEN,
                runtime_args! {
                    ARG_TOKEN_ID => token_hash,
                    ARG_TO => to
                },
            ),
        };
    }
}
