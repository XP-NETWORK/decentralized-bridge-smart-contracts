// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface INFTStorageDeployer {
    /**
     * @dev Get the owner of the contract.
     *
     * @return The address of the owner.
     */
    function owner() external view returns (address);

    /**
     * @dev Set a new owner for the contract.
     *
     * @param _owner The address to be set as the new owner.
     */
    function setOwner(address _owner) external;

    /**
     * @dev Deploy a new NFTStorageERC721 contract with the specified collection address.
     *
     * @param collectionAddress The address of the ERC721 collection.
     * @return The address of the newly deployed NFTStorageERC721 contract.
     */
    function deployNFT721Storage(
        address collectionAddress
    ) external returns (address);
}
