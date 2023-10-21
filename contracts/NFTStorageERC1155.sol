// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC1155/utils/ERC1155Holder.sol";
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

contract NFTStorageERC1155 is ERC1155Holder {
    address public owner;
    IERC1155 public collectionAddress;

    constructor(address _collectionAddress, address _owner) {
        owner = _owner;
        collectionAddress = IERC1155(_collectionAddress);
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Only the owner can call this function");
        _;
    }

    modifier onlyValidator() {
        require(false, "Only validators can unlock token");
        _;
    }

    // Function to deposit an ERC-1155 token into this contract
    function depositToken(uint256 tokenId, uint256 amount) external onlyOwner {
        // Transfer the token to this contract
        collectionAddress.safeTransferFrom(
            msg.sender,
            address(this),
            tokenId,
            amount,
            ""
        );
    }

    // Function to allow the owner of this contract to transfer an ERC-1155 token to another address
    function unlockToken(
        uint256 tokenId,
        uint256 amount
    ) external onlyOwner onlyValidator {
        // Ensure this contract has the balance before transferring
        require(
            collectionAddress.balanceOf(address(this), tokenId) >= amount,
            "Insufficient token balance"
        );
        collectionAddress.safeTransferFrom(
            address(this),
            owner,
            tokenId,
            amount,
            ""
        );
    }
}
