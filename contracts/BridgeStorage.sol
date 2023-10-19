// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @dev Struct representing details of an NFT transfer between chains.
 */
struct NftTransferDetails {
    uint256 id; // Unique ID for the NFT transfer
    string sourceChain; // Chain from where the NFT is being transferred
    string destinationChain; // Chain to where the NFT is being transferred
    string sourceUserAddress; // User's address in the source chain
    string destinationUserAddress; // User's address in the destination chain
    string sourceNftContractAddress; // Address of the NFT contract in the source chain
    string metadata; // Metadata related to the NFT being transferred
    string transactionHash; // Transaction hash of the transfer on the source chain
    uint256 count; // Number of NFTs being transferred
    string nftType; // Type of the NFT (could be ERC721 or ERC1155)
}

/**
 * @dev Struct representing an NFT transfer with associated signatures.
 */
struct NftTransferWithSignatures {
    NftTransferDetails transferDetails; // Details of the NFT transfer
    string[] signatures; // Signatures associated with the transfer
}

struct ChainFee {
    string chain;
    uint256 fee;
}

/**
 * @title BridgeStorage
 * @dev A contract to store and manage cross-chain transfer signatures and details.
 */
contract BridgeStorage {
    // Current epoch for each chain
    mapping(string => uint256) public chainEpoch;

    // Current epoch for each validator
    mapping(address => uint256) public validatorEpoch;

    // Mapping from staker's address to an array of their signatures.
    mapping(address => string[]) public stakingSignatures;

    // Mapping of existing validators.
    mapping(address => bool) public validators;

    // Mapping of votes of new validators. validatorStatusChangeVotes[validatorAddress][senderAddress][status][validatorEpoch] = numberOfVotes
    mapping(address => mapping(address => mapping(bool => mapping(uint256 => uint256))))
        public validatorStatusChangeVotes;

    // Mapping to check if already voted on newValidator  validatorVoted[validatorAddress][senderAddress][validatorEpoch] = true/ false
    mapping(address => mapping(address => mapping(uint256 => bool)))
        public validatorVoted;

    // Mapping of votes of new chain fee. chainFeeVotes[chain][fee][chainEpoch] = votes
    mapping(string => mapping(uint256 => mapping(uint256 => uint256)))
        public chainFeeVotes;

    // Mapping of to check if already voted on a chain fee. chainFeeVotes[chain][fee][address][chainEpoch] = true/false
    mapping(string => mapping(uint256 => mapping(address => mapping(uint256 => bool))))
        public chainFeeVoted;

    // total validator count
    uint256 public validatorCount;

    // Mapping from a concatenated string of chain and transaction hash to the transfer details and its signatures.
    mapping(string => NftTransferWithSignatures) public lockSignatures;

    // Mapping to check if a signature has already been used.
    mapping(string => bool) public usedSignatures;

    // Mapping to store fee for all chains
    mapping(string => uint256) public chainFee;

    /**
     * @dev bootstrap the bridge storage with a intial validator and intial fee for chains
     * @param bootstrapValidator Address of the staker.
     * @param bootstrapChainFee array of chain fee
     */
    constructor(
        address bootstrapValidator,
        ChainFee[] memory bootstrapChainFee
    ) {
        validators[bootstrapValidator] = true;
        validatorCount++;

        for (uint256 i = 0; i < bootstrapChainFee.length; i++) {
            chainFee[bootstrapChainFee[i].chain] = bootstrapChainFee[i].fee;
        }
    }

    /**
     * @dev Concatenates two strings.
     * @param a First string.
     * @param b Second string.
     * @return Concatenated string.
     */
    function concatenate(
        string calldata a,
        string calldata b
    ) public pure returns (string memory) {
        return string(abi.encodePacked(a, b));
    }

    /**
     * @dev modifier to check if caller is validator
     */
    modifier onlyValidator() {
        require(
            validators[msg.sender],
            "Only validators can call this function"
        );
        _;
    }

    /**
     * @dev change / add a chain with new fee
     * @param chain  new / old chain.
     * @param fee  new fee.
     */
    function changeChainFee(
        string calldata chain,
        uint256 fee
    ) public onlyValidator {
        require(
            chainFeeVoted[chain][fee][msg.sender][chainEpoch[chain]] == false,
            "Already voted"
        );
        chainFeeVoted[chain][fee][msg.sender][chainEpoch[chain]] = true;

        chainFeeVotes[chain][fee][chainEpoch[chain]]++;

        uint256 twoByThreeValidators = (2 * validatorCount) / 3;

        if (
            chainFeeVotes[chain][fee][chainEpoch[chain]] >=
            twoByThreeValidators + 1
        ) {
            chainFee[chain] = fee;
            chainEpoch[chain]++;
        }
    }
    
    /**
     * @dev change status of a validator with  2/3 + 1 of total validators votes are given
     * @param _validatorAddress new validator address.
     */
    function changeValidatorStatus(
        address _validatorAddress,
        bool status
    ) public onlyValidator {
        uint256 _validatorEpoch = validatorEpoch[_validatorAddress];
        require(
            validatorVoted[_validatorAddress][msg.sender][_validatorEpoch] ==
                false,
            "Already voted for this validator"
        );

        validatorVoted[_validatorAddress][msg.sender][_validatorEpoch] = true;

        validatorStatusChangeVotes[_validatorAddress][msg.sender][status][
            _validatorEpoch
        ]++;

        uint256 twoByThreeValidators = (2 * validatorCount) / 3;

        if (
            (validatorStatusChangeVotes[_validatorAddress][msg.sender][status][
                _validatorEpoch
            ] >= twoByThreeValidators + 1)
        ) {
            if (status && validators[_validatorAddress] == false)
                validatorCount++;
            else if (status == false && validators[_validatorAddress])
                validatorCount--;
            validators[_validatorAddress] = status;
            validatorEpoch[_validatorAddress]++;
        }
    }

    /**
     * @dev Approves a stake using a signature.
     * @param stakerAddress Address of the staker.
     * @param signature The signature to be approved.
     */
    function approveStake(
        address stakerAddress,
        string calldata signature
    ) public onlyValidator {
        require(!usedSignatures[signature], "Signature already used");
        usedSignatures[signature] = true;
        stakingSignatures[stakerAddress].push(signature);
        changeValidatorStatus(stakerAddress, true);
    }

    /**
     * @dev Retrieves staking signatures for a specific staker.
     * @param stakerAddress Address of the staker.
     * @return Array of signatures.
     */
    function getStakingSignatures(
        address stakerAddress
    ) external view returns (string[] memory) {
        return stakingSignatures[stakerAddress];
    }

    /**
     * @dev Retrieves the count of staking signatures for a specific staker.
     * @param stakerAddress Address of the staker.
     * @return Number of signatures.
     */
    function getStakingSignaturesCount(
        address stakerAddress
    ) external view returns (uint256) {
        return stakingSignatures[stakerAddress].length;
    }

    /**
     * @dev Approves the locking of an NFT using a signature and associated transfer details.
     * @param nftTransferDetails Details of the NFT transfer.
     * @param signature The signature to be approved.
     */
    function approveLockNft(
        NftTransferDetails calldata nftTransferDetails,
        string calldata signature
    ) public onlyValidator {
        require(!usedSignatures[signature], "Signature already used");
        usedSignatures[signature] = true;

        string memory chainAndTxHash = concatenate(
            nftTransferDetails.sourceChain,
            nftTransferDetails.transactionHash
        );

        if (
            bytes(
                lockSignatures[chainAndTxHash].transferDetails.transactionHash
            ).length == 0
        ) {
            lockSignatures[chainAndTxHash] = NftTransferWithSignatures({
                transferDetails: nftTransferDetails,
                signatures: new string[](0)
            });
        }

        lockSignatures[chainAndTxHash].signatures.push(signature);
    }

    /**
     * @dev Retrieves lock signatures and associated transfer details for a specific chain and transaction.
     * @param chain Source chain of the transfer.
     * @param txHash Transaction hash of the transfer.
     * @return Struct containing transfer details and associated signatures.
     */
    function getLockNftSignatures(
        string calldata chain,
        string calldata txHash
    ) external view returns (NftTransferWithSignatures memory) {
        string memory chainAndTxHash = concatenate(chain, txHash);
        return lockSignatures[chainAndTxHash];
    }

    /**
     * @dev Retrieves the count of lock signatures for a specific chain and transaction.
     * @param chain Source chain of the transfer.
     * @param txHash Transaction hash of the transfer.
     * @return Number of signatures.
     */
    function getLockNftSignaturesCount(
        string calldata chain,
        string calldata txHash
    ) external view returns (uint256) {
        string memory chainAndTxHash = concatenate(chain, txHash);
        return lockSignatures[chainAndTxHash].signatures.length;
    }
}
