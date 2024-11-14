// RUNTIME ARGS INITIALIZE
pub const ARG_VALIDATORS: &str = "bootstrap_validator_arg";
pub const ARG_CHAIN_TYPE: &str = "chain_type_arg";
pub const ARG_COLLECTION_DEPLOYER: &str = "collection_deployer_arg";
pub const ARG_STORAGE_DEPLOYER: &str = "storage_deployer_arg";

// RUNTIME ARGS ADD VALIDATOR
pub const ARG_NEW_VALIDATOR_PUBLIC_KEY: &str = "new_validator_public_key_arg";
pub const ARG_SIGNATURES: &str = "signatures_arg";

// RUNTIME ARGS LOCK
pub const ARG_TOKEN_ID: &str = "token_id_arg";
pub const ARG_DESTINATION_CHAIN: &str = "destination_chain_arg";
pub const ARG_DESTINATION_USER_ADDRESS: &str = "destination_user_address_arg";
pub const ARG_SOURCE_NFT_CONTRACT_ADDRESS: &str = "source_nft_contract_address_arg";
pub const ARG_METADATA_URI: &str = "metadata_uri_arg";

// CONTRACT STATE
pub const INITIALIZED: &str = "initialized";
pub const THIS_CONTRACT: &str = "bridge_contract";
pub const INSTALLER: &str = "installer";
pub const KEY_PURSE: &str = "bridge_purse";
pub const HASH_KEY_NAME: &str = "bridge_package";
pub const ACCESS_KEY_NAME: &str = "access_key_name_bridge";
pub const KEY_CHAIN_TYPE: &str = "chain_type";
pub const KEY_COLLECTION_DEPLOYER: &str = "collection_deployer";
pub const KEY_STORAGE_DEPLOYER: &str = "storage_deployer";
pub const KEY_VALIDATORS_COUNT: &str = "validators_count";

// DICTIONARIES
pub const KEY_VALIDATORS_DICT: &str = "validators_dict";
pub const KEY_BLACKLIST_VALIDATORS_DICT: &str = "blacklist_validators_dict";
pub const KEY_UNIQUE_IDENTIFIERS_DICT: &str = "unique_identifiers_dict";
pub const KEY_ORIGINAL_TO_DUPLICATE_DICT: &str = "original_to_duplicate_dict";
pub const KEY_DUPLICATE_TO_ORIGINAL_DICT: &str = "duplicate_to_original_dict";
pub const KEY_ORIGINAL_STORAGE_DICT: &str = "original_storage_dict";
pub const KEY_DUPLICATE_STORAGE_DICT: &str = "duplicate_storage_dict";
