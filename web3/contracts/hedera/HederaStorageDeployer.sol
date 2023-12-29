// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./HederaNFTStorage.sol";
import "./interfaces/INFTStorageDeployer.sol";

contract HederaNFTStorageDeployer is INFTStorageDeployer {
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
        HederaNFTStorage newStorageAddress = new HederaNFTStorage(
            collectionAddress,
            owner
        );

        return address(newStorageAddress);
    }
}
