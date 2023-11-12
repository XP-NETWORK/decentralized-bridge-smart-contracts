// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface INFTStorageERC1155 {
    /**
     * @dev Get the owner of the contract.
     *
     * @return The address of the owner.
     */
    function owner() external view returns (address);

    /**
     * @dev Get the associated ERC1155 collection address.
     *
     * @return The address of the ERC1155 collection.
     */
    function collectionAddress() external view returns (address);

    /**
     * @dev Deposit an ERC-1155 token into this contract.
     *
     * @param tokenId The ID of the token to be deposited.
     * @param amount The amount of the token to be deposited.
     */
    function depositToken(uint256 tokenId, uint256 amount) external;

    /**
     * @dev Allow the owner of this contract to transfer an ERC-1155 token to another address.
     *
     * @param tokenId The ID of the token to be unlocked.
     * @param amount The amount of the token to be unlocked.
     */
    function unlockToken(uint256 tokenId, uint256 amount, address to) external;
}
