// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./ERC1155Royalty.sol";
import "./ERC721Royalty.sol";
import "hardhat/console.sol";

contract NFTCollectionDeployer {
    address public owner;
    address public bridge;

    constructor() {}

    modifier onlyBridge() {
        console.log("msg.sender",msg.sender);
        console.log("bridge",bridge);
        console.log(msg.sender == bridge, "condition");
        require(msg.sender == bridge, "Not bridge!");
        _;
    }

    event CreatedCollection(address collectionAddress);

    function setOwner(address _owner, address _bridge) external {
        require(owner == address(0), "Owner already set!");
        require(bridge == address(0), "Bridge already set!");
        owner = _owner;
        bridge = _bridge;
    }

    function deployNFT721Collection(
        string memory name,
        string memory symbol
    ) external onlyBridge() returns (address) {
        ERC721Royalty newERC721CollectionAddress = new ERC721Royalty(
            name,
            symbol,
            owner,
            msg.sender
        );

        address collectionAddress = address(newERC721CollectionAddress);

        emit CreatedCollection(collectionAddress);

        return collectionAddress;
    }

    function deployNFT1155Collection() external onlyBridge returns (address) {
        ERC1155Royalty newERC1155CollectionAddress = new ERC1155Royalty(owner, msg.sender);

        address collectionAddress = address(newERC1155CollectionAddress);

        emit CreatedCollection(collectionAddress);

        return collectionAddress;
    }
}
