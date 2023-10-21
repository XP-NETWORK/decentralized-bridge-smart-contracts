// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface INFTStorageERC721 {
    /**
     * @dev Get the owner of the contract.
     *
     * @return The address of the owner.
     */
    function owner() external view returns (address);

    /**
     * @dev Get the associated ERC721 collection address.
     *
     * @return The address of the ERC721 collection.
     */
    function collectionAddress() external view returns (address);

    /**
     * @dev Deposit an ERC-721 token into this contract.
     *
     * @param tokenId The ID of the token to be deposited.
     */
    function depositToken(uint256 tokenId) external;

    /**
     * @dev Allow the owner of this contract to transfer an ERC-721 token to another address.
     *
     * @param tokenId The ID of the token to be unlocked.
     */
    function unlockToken(uint256 tokenId, address to) external;
}
