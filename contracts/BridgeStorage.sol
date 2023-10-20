// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

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

/**
 * @title BridgeStorage
 * @dev A contract to store and manage cross-chain transfer signatures and details.
 */
contract BridgeStorage {
    
    // Mapping from staker's address to an array of their signatures.
    mapping(address => string[]) public stakingSignatures;
    
    // Mapping from a concatenated string of chain and transaction hash to the transfer details and its signatures.
    mapping(string => NftTransferWithSignatures) public lockSignatures;

    // Mapping to check if a signature has already been used.
    mapping(string => bool) public usedSignatures;

    /**
     * @dev Concatenates two strings.
     * @param a First string.
     * @param b Second string.
     * @return Concatenated string.
     */
    function concatenate(string memory a, string memory b) public pure returns (string memory) {
        return string(abi.encodePacked(a, b));
    }

    /**
     * @dev Approves a stake using a signature.
     * @param stakerAddress Address of the staker.
     * @param signature The signature to be approved.
     */
    function approveStake(address stakerAddress, string calldata signature) public {
        require(!usedSignatures[signature], "Signature already used");
        usedSignatures[signature] = true;
        stakingSignatures[stakerAddress].push(signature);
    }

    /**
     * @dev Retrieves staking signatures for a specific staker.
     * @param stakerAddress Address of the staker.
     * @return Array of signatures.
     */
    function getStakingSignatures(address stakerAddress) external view returns (string[] memory) {
        return stakingSignatures[stakerAddress];
    }

    /**
     * @dev Retrieves the count of staking signatures for a specific staker.
     * @param stakerAddress Address of the staker.
     * @return Number of signatures.
     */
    function getStakingSignaturesCount(address stakerAddress) external view returns (uint256) {
        return stakingSignatures[stakerAddress].length;
    }

    /**
     * @dev Approves the locking of an NFT using a signature and associated transfer details.
     * @param nftTransferDetails Details of the NFT transfer.
     * @param signature The signature to be approved.
     */
    function approveLockNft(NftTransferDetails calldata nftTransferDetails, string calldata signature) public {
        require(!usedSignatures[signature], "Signature already used");
        usedSignatures[signature] = true;

        string memory chainAndTxHash = concatenate(nftTransferDetails.sourceChain, nftTransferDetails.transactionHash);

        if (bytes(lockSignatures[chainAndTxHash].transferDetails.transactionHash).length == 0) {
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
    function getLockNftSignatures(string calldata chain, string calldata txHash) external view returns (NftTransferWithSignatures memory) {
        string memory chainAndTxHash = concatenate(chain, txHash);
        return lockSignatures[chainAndTxHash];
    }

    /**
     * @dev Retrieves the count of lock signatures for a specific chain and transaction.
     * @param chain Source chain of the transfer.
     * @param txHash Transaction hash of the transfer.
     * @return Number of signatures.
     */
    function getLockNftSignaturesCount(string calldata chain, string calldata txHash) external view returns (uint256) {
        string memory chainAndTxHash = concatenate(chain, txHash);
        return lockSignatures[chainAndTxHash].signatures.length;
    }
}
