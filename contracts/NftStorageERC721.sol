// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/utils/ERC721Holder.sol";
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";

contract NftStorageERC721 is ERC721Holder {

    address public owner;

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Only the owner can call this function");
        _;
    }

    // Function to deposit an ERC-721 token into this contract
    function depositToken(address tokenAddress, uint256 tokenId) external {
        IERC721 token = IERC721(tokenAddress);
        // Ensure the msg.sender is the owner of the token
        // require(token.ownerOf(tokenId) == msg.sender, "You are not the owner of this token");
        // Transfer the token to this contract
        token.safeTransferFrom(msg.sender, address(this), tokenId);
    }

    // Function to allow the owner of this contract to transfer an ERC-721 token to another address
    function transferToken(address tokenAddress, uint256 tokenId, address to) external onlyOwner {
        IERC721 token = IERC721(tokenAddress);
        // Ensure this contract is the owner of the token before transferring
        require(token.ownerOf(tokenId) == address(this), "This contract is not the owner of this token");
        token.safeTransferFrom(address(this), to, tokenId);
    }
}
