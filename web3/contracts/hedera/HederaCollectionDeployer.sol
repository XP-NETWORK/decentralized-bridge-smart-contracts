// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./HederaNFT.sol";
import "./HederaNFTClaim.sol";
import "./interfaces/INFTCollectionDeployer.sol";

contract NFTCollectionDeployer is INFTCollectionDeployer {
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
        string memory symbol,
        int64 royaltyNum,
        address royaltyReceiver
    ) external onlyOwner returns (address) {
        HederaNFTClaim claimContract = new HederaNFTClaim();
        HederaNft newERC721CollectionAddress = new HederaNft{value: 25 ether}(
            name,
            symbol,
            owner,
            claimContract,
            royaltyNum,
            royaltyReceiver
        );
        address collectionAddress = address(newERC721CollectionAddress);
        claimContract.transferOwnership(collectionAddress);

        emit CreatedCollection(collectionAddress);

        return collectionAddress;
    }

    function deployNFT721Collection(
        string memory name,
        string memory symbol
    ) external override returns (address) {}
}
