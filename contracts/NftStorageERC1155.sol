// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC1155/utils/ERC1155Holder.sol";
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

contract NftStorageERC1155 is ERC1155Holder {

    address public owner;

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Only the owner can call this function");
        _;
    }

    // Function to deposit an ERC-1155 token into this contract
    function depositToken(address tokenAddress, uint256 tokenId, uint256 amount) external onlyOwner {
        IERC1155 token = IERC1155(tokenAddress);
        // Transfer the token to this contract
        token.safeTransferFrom(msg.sender, address(this), tokenId, amount, "");
    }

    // Function to allow the owner of this contract to transfer an ERC-1155 token to another address
    function transferToken(address tokenAddress, uint256 tokenId, address to, uint256 amount) external onlyOwner {
        IERC1155 token = IERC1155(tokenAddress);
        // Ensure this contract has the balance before transferring
        require(token.balanceOf(address(this), tokenId) >= amount, "Insufficient token balance");
        token.safeTransferFrom(address(this), to, tokenId, amount, "");
    }
}