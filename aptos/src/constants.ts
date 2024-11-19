const BRIDGE_ADDRESS =
  "41f82a0bf778301b06a4ec405275d70932ec8373473b963c80564d6d8d0432f5";

const BRIDGE_MODULE = "aptos_nft_bridge";
const MINT_MODULE = "mint";

const BRIDGE_FUNCTIONS = {
  Initialize: "initialize",
  MoveBridgeResource: "move_bridge_resource",
  AddValidator: "add_validator",
  Lock721: "lock_721",
  Lock1155: "lock_1155",
  Claim721: "claim_721",
  Claim1155: "claim_1155",
  OwnsNFT: "owns_nft",
  ClaimValidatorRewards: "claim_validator_rewards",
};

const MINT_FUNCTIONS = {
  MINT_TO: "mint_to",
  MINT_1155_TO: "mint_1155_to",
};

enum CONTRACT_ERROR_CODES {
  E_ALREADY_INITIALIZED = "E_ALREADY_INITIALIZED",
  E_NOT_BRIDGE_ADMIN = "E_NOT_BRIDGE_ADMIN",
  E_VALIDATORS_LENGTH_ZERO = "E_VALIDATORS_LENGTH_ZERO",
  E_VALIDATOR_ALREADY_EXIST = "E_VALIDATOR_ALREADY_EXIST",
  E_THERSHOLD_NOT_REACHED = "E_THERSHOLD_NOT_REACHED",
  E_INVALID_GK = "E_INVALID_GK",
  E_INVALID_SIGNATURE = "E_INVALID_SIGNATURE",
  E_NOT_INITIALIZED = "E_NOT_INITIALIZED",
  E_INVALID_FEE = "E_INVALID_FEE",
  E_NO_REWARDS_AVAILABLE = "E_NO_REWARDS_AVAILABLE",
  E_INVALID_DESTINATION_CHAIN = "E_INVALID_DESTINATION_CHAIN",
  E_INVALID_NFT_TYPE = "E_INVALID_NFT_TYPE",
  E_SIGNATURES_PUBLIC_KEYS_LENGTH_NOT_SAME = "E_SIGNATURES_PUBLIC_KEYS_LENGTH_NOT_SAME",
  EOBJECT_DOES_NOT_EXIST = "EOBJECT_DOES_NOT_EXIST",
  E_TOKEN_AMOUNT_IS_ZERO = "E_TOKEN_AMOUNT_IS_ZERO",
  E_CLAIM_ALREADY_PROCESSED = "E_CLAIM_ALREADY_PROCESSED",
  EINSUFFICIENT_BALANCE = "EINSUFFICIENT_BALANCE",
  E_VALIDATOR_DOESNOT_EXIST = "E_VALIDATOR_DOESNOT_EXIST",
  E_VALIDATOR_PENDING_REWARD_IS_ZERO = "E_VALIDATOR_PENDING_REWARD_IS_ZERO",
  ENOT_OBJECT_OWNER = "ENOT_OBJECT_OWNER",
  E_DESTINATION_CHAIN_SAME_AS_SOURCE = "E_DESTINATION_CHAIN_SAME_AS_SOURCE",
}

const CHAIN_ID = "APTOS";

const CLAIM_FEE_20_APT = 20 * 10 ** 8;
const CLAIM_FEE_POINT_1_APT = 0.1 * 10 ** 8;

export {
  BRIDGE_ADDRESS,
  BRIDGE_MODULE,
  BRIDGE_FUNCTIONS,
  CONTRACT_ERROR_CODES,
  CHAIN_ID,
  MINT_MODULE,
  MINT_FUNCTIONS,
  CLAIM_FEE_20_APT,
  CLAIM_FEE_POINT_1_APT,
};
