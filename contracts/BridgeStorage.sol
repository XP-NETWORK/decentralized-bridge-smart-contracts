// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

struct NftTransferDetails {
    uint256 id;
    string sourceChain;
    string destinationChain;
    string sourceUserAddress;
    string destinationUserAddress;
    string sourceNftContractAddress;
    string metadata;
    string transactionHash;
    uint256 count;
    string nftType;
}

struct NftTransferWithSignatures {
    NftTransferDetails transferDetails;
    string[] signatures;
}


contract BridgeStorage {
    
    mapping(string => string[]) public stakingSignatures; // Chain_TxHash => Signatures []
    mapping(string=> NftTransferWithSignatures ) public lockSignatures; // Chain_TxHash => Signatures []

    mapping(string => bool) public usedSignatures; // Signature => Used / Unused


    function concatenate(string memory a, string memory b) public pure returns (string memory) {
        return string(abi.encodePacked(a, b));
    }


    function approveStake( string calldata txHash, string calldata chain,string calldata signature) public  {
        require(usedSignatures[signature] != true, "Signature already used");
        usedSignatures[signature] = true;
        stakingSignatures[concatenate(txHash, chain)].push(signature);
    }

    function getStakingSignatures ( string calldata txHash, string calldata chain) external view returns ( string[] memory signatures ) {
        return  stakingSignatures[concatenate(txHash, chain)];
    } 

    function getStakingSignaturesCount ( string calldata txHash, string calldata chain) external view returns ( uint256 signaturesCount ) {
        return  stakingSignatures[concatenate(txHash, chain)].length;
    } 

    function approveLockNft(NftTransferDetails calldata nftTransferDetails, string calldata signature) public {
        require(usedSignatures[signature] != true, "Signature already used");
        usedSignatures[signature] = true;

         if (bytes(lockSignatures[nftTransferDetails.transactionHash].transferDetails.transactionHash).length == 0) {
            // If not, initialize it with the provided transferDetails and an empty signatures array
            lockSignatures[concatenate(nftTransferDetails.transactionHash, nftTransferDetails.sourceChain)] = NftTransferWithSignatures({
                transferDetails: nftTransferDetails,
                signatures: new string[](0)
            });
        }
        
        lockSignatures[concatenate(nftTransferDetails.transactionHash, nftTransferDetails.sourceChain)].signatures.push(signature);
    }

    function getLockNftSignatures ( string calldata txHash, string calldata chain) external view returns ( NftTransferWithSignatures memory signaturesWithTransferDetails ) {
        return  lockSignatures[concatenate(txHash, chain)];
    } 

    function getLockNftSignaturesCount ( string calldata txHash, string calldata chain) external view returns ( uint256 signaturesCount ) {
        return  lockSignatures[concatenate(txHash, chain)].signatures.length;
    } 
    
}