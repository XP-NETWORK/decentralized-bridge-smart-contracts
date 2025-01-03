pub const ARG_DATA_HASH: &str = "data_hash_arg";
pub const ARG_DATA_TYPE: &str = "data_type_arg";
pub const ARG_SIGNATURES: &str = "signatures_arg";
// RUNTIME ARGS INITIALIZE
pub const ARG_VALIDATOR: &str = "bootstrap_validator_arg";
pub const ARG_CHAIN_TYPE: &str = "chain_type_arg";
pub const ARG_SERVICE_ADDRESS: &str = "service_address_arg";
pub const ARG_SELF_HASH: &str = "self_hash_arg";
pub const ARG_STORAGE_DEPLOY_FEE: &str = "storage_deploy_fee_arg";
pub const ARG_COLLECTION_DEPLOY_FEE: &str = "collection_deploy_fee_arg";

// RUNTIME ARGS ADD VALIDATOR
pub const ARG_NEW_VALIDATOR_PUBLIC_KEY: &str = "new_validator_public_key_arg";

// RUNTIME ARGS BLACKLIST VALIDATOR
pub const ARG_VALIDATOR_PUBLIC_KEY: &str = "validator_public_key_arg";

// RUNTIME ARGS LOCK
pub const ARG_TOKEN_ID: &str = "token_id_arg";
pub const ARG_DESTINATION_CHAIN: &str = "destination_chain_arg";
pub const ARG_DESTINATION_USER_ADDRESS: &str = "destination_user_address_arg";
pub const ARG_SOURCE_NFT_CONTRACT_ADDRESS: &str = "source_nft_contract_address_arg";
pub const ARG_SENDER_PURSE: &str = "sender_purse";
pub const ARG_AMOUNT: &str = "amount";

pub const ARG_STORAGE_ADDRESS: &str = "storage_address_arg";

// RUNTIME ARGS CLAIM
pub const ARG_SOURCE_CHAIN: &str = "source_chain_arg";
pub const ARG_NAME: &str = "name_arg";
pub const ARG_SYMBOL: &str = "symbol_arg";
pub const ARG_ROYALTY: &str = "royalty_arg";
pub const ARG_ROYALTY_RECEIVER: &str = "royalty_receiver_arg";
pub const ARG_METADATA: &str = "metadata_arg";
pub const ARG_TRANSACTION_HASH: &str = "transaction_hash_arg";
pub const ARG_TOKEN_AMOUNT: &str = "token_amount_arg";
pub const ARG_NFT_TYPE: &str = "nft_type_arg";
pub const ARG_FEE: &str = "fee_arg";
pub const ARG_LOCK_TX_CHAIN: &str = "lock_tx_chain_arg";
pub const ARG_COLLECTION_ADDRESS: &str = "collection_address_arg";

// INSTALLER STATE
pub const THIS_CONTRACT: &str = "bridge_contract_";

// CONTRACT STATE
pub const INITIALIZED: &str = "initialized";
pub const INSTALLER: &str = "installer";
pub const KEY_PURSE: &str = "bridge_purse";
pub const HASH_KEY_NAME: &str = "bridge_package";
pub const ACCESS_KEY_NAME: &str = "access_key_name_bridge";
pub const KEY_CHAIN_TYPE: &str = "chain_type";
pub const KEY_VALIDATORS_COUNT: &str = "validators_count";
pub const KEY_STORAGE_DEPLOY_FEE_NONCE: &str = "storage_deploy_fee_nonce";
pub const KEY_COLLECTION_DEPLOY_FEE_NONCE: &str = "collection_deploy_fee_nonce";
pub const KEY_SERVICE_ADDRESS: &str = "service_address";
pub const KEY_TYPE_ERC721: &str = "nft_type";
pub const KEY_SELF_HASH: &str = "self_hash";

pub const KEY_STORAGE_DEPLOY_FEE: &str = "storage_deploy_fee";
pub const KEY_COLLECTION_DEPLOY_FEE: &str = "collection_deploy_fee";

// DICTIONARIES
pub const KEY_VALIDATORS_DICT: &str = "validators_dict";
pub const KEY_BLACKLIST_VALIDATORS_DICT: &str = "blacklist_validators_dict";
pub const KEY_ORIGINAL_TO_DUPLICATE_DICT: &str = "original_to_duplicate_dict";
pub const KEY_DUPLICATE_TO_ORIGINAL_DICT: &str = "duplicate_to_original_dict";
pub const KEY_ORIGINAL_STORAGE_DICT: &str = "original_storage_dict";
pub const KEY_DUPLICATE_STORAGE_DICT: &str = "duplicate_storage_dict";
pub const KEY_SUBMITTED_SIGNATURES_DICT: &str = "submitted_signatures_dict";
pub const KEY_TOKEN_INFO_GET_SELF_DICT: &str = "token_info_get_self_dict";
pub const KEY_TOKEN_INFO_KEY_SELF_DICT: &str = "token_info_key_self_dict";
pub const KEY_WAITING_DICT: &str = "waiting_dict";
