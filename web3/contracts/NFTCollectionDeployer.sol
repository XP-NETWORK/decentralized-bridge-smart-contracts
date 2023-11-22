// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./ERC1155Royalty.sol";
import "./ERC721Royalty.sol";

contract NFTCollectionDeployer {
    address public owner;

    constructor() {}

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call");
        _;
    }

    event CreatedCollection(address collectionAddress);

    function setOwner(address _owner) external {
        require(owner == address(0), "Owner already set!");
        owner = _owner;
    }

    function deployNFT721Collection(
        string memory name,
        string memory symbol
    ) external onlyOwner returns (address) {
        ERC721Royalty newERC721CollectionAddress = new ERC721Royalty(
            name,
            symbol,
            owner
        );

        address collectionAddress = address(newERC721CollectionAddress);

        emit CreatedCollection(collectionAddress);

        return collectionAddress;
    }

    function deployNFT1155Collection() external onlyOwner returns (address) {
        ERC1155Royalty newERC1155CollectionAddress = new ERC1155Royalty(owner);

        address collectionAddress = address(newERC1155CollectionAddress);

        emit CreatedCollection(collectionAddress);

        return collectionAddress;
    }
}
