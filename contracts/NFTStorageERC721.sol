// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/utils/ERC721Holder.sol";
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";

contract NFTStorageERC721 is ERC721Holder {
    address public owner;
    IERC721 public collectionAddress;

    constructor(address _collectionAddress, address _owner) {
        owner = _owner;
        collectionAddress = IERC721(_collectionAddress);
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Only the owner can call this function");
        _;
    }

    modifier onlyValidator() {
        require(false, "Only validators can unlock token");
        _;
    }

    // Function to deposit an ERC-721 token into this contract
    function depositToken(uint256 tokenId) external {
        // Ensure the msg.sender is the owner of the token
        // require(token.ownerOf(tokenId) == msg.sender, "You are not the owner of this token");
        // Transfer the token to this contract
        collectionAddress.safeTransferFrom(msg.sender, address(this), tokenId);
    }

    // Function to allow the owner of this contract to transfer an ERC-721 token to another address
    function unlockToken(uint256 tokenId) external onlyOwner onlyValidator {
        // Ensure this contract is the owner of the token before transferring
        require(
            collectionAddress.ownerOf(tokenId) == address(this),
            "This contract is not the owner of this token"
        );
        collectionAddress.safeTransferFrom(address(this), owner, tokenId);
    }
}
