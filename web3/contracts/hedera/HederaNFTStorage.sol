// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../../lib/hedera/contracts/hts-precompile/HederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/IHederaTokenService.sol";
import "./interfaces/IHRC.sol";
import "./interfaces/INFTClaim.sol";
import "./HederaNFTClaim.sol";
import "./interfaces/IHTSCompatabilityLayer.sol";

contract HederaNFTStorage is HederaTokenService {
    address public owner;
    address public collectionAddress;
    INFTClaim public claimContract;
    event Transfer(
        address indexed from,
        address indexed to,
        int64 indexed tokenId
    );

    constructor(address _collectionAddress, address _owner) {
        owner = _owner;
        collectionAddress = _collectionAddress;
        IHRC(collectionAddress).associate();
        claimContract = new HederaNFTClaim();
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Only the owner can call this function");
        _;
    }

    // Function to deposit an ERC-721 token into this contract
    function depositToken(uint256 tokenId) external {
        int response = transferNFT(
            collectionAddress,
            msg.sender,
            address(this),
            int64(uint64(tokenId))
        );
        require(response == 22, "Failed to transfer NFT");
    }

    // Function to allow the owner of this contract to transfer an ERC-721 token to another address. We might need claiming functionality.
    function unlockToken(uint256 tokenId, address to) external onlyOwner {
        // Ensure this contract is the owner of the token before transferring
        IHTSCompatibilityLayer htsCl = IHTSCompatibilityLayer(collectionAddress);
        require(
            htsCl.ownerOf(tokenId) == address(this),
            "This contract is not the owner of this token"
        );
        int tres = transferNFT(
            collectionAddress,
            address(this),
            to,
            int64(uint64(tokenId))
        );
        if (tres == 184) {
            // The NFT Contract is not associated to the HTS.
            claimContract.addClaimRecord(
                to,
                collectionAddress,
                int64(uint64(tokenId))
            );
        }
        require(tres == 22, "Failed to transfer NFT");
    }

    function _transferClaim(
        address to,
        int64 serialNum,
        address token
    ) private returns (int256) {
        int256 resp = transferNFT(token, address(this), to, serialNum);
        return resp;
    }

    function claimNft(int64 serialNum, address token) external {
        claimContract.getClaimRecord(msg.sender, token, serialNum);
        int256 resp = _transferClaim(msg.sender, serialNum, token);
        require(
            resp == HederaResponseCodes.SUCCESS,
            "Failed to transfer token"
        );
        emit Transfer(address(this), msg.sender, serialNum);
    }
}
