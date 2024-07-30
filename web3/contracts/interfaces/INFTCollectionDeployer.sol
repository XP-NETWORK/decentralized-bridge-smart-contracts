// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface INFTCollectionDeployer {
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
    function setOwner(address _owner, address _bridge) external;

    /**
     * @dev Deploy a new ERC721Royalty contract with the specified name and symbol.
     *
     * @param name The name of the new ERC721Royalty contract.
     * @param symbol The symbol of the new ERC721Royalty contract.
     * @return The address of the newly deployed ERC721Royalty contract.
     */
    function deployNFT721Collection(
        string memory name,
        string memory symbol
    ) external returns (address);

    /**
     * @dev Deploy a new ERC1155Royalty contract.
     *
     * @return The address of the newly deployed ERC1155Royalty contract.
     */
    function deployNFT1155Collection() external returns (address);
}
