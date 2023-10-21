// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./NFTStorageERC721.sol";
import "./NFTStorageERC1155.sol";

contract NFTStorageDeployer {
    address public owner;

    constructor() {}

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call");
        _;
    }

    function setOwner(address _owner) external {
        require(owner == address(0), "Owner already set!");
        owner = _owner;
    }

    function deployNFT721Storage(
        address collectionAddress
    ) external onlyOwner returns (address) {
        NFTStorageERC721 newStorageAddress = new NFTStorageERC721(
            collectionAddress,
            owner
        );

        return address(newStorageAddress);
    }

    function deployNFT1155Storage(
        address collectionAddress
    ) external onlyOwner returns (address) {
        NFTStorageERC1155 newStorageAddress = new NFTStorageERC1155(
            collectionAddress,
            owner
        );

        return address(newStorageAddress);
    }
}

