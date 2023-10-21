// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import "@openzeppelin/contracts/token/common/ERC2981.sol";

interface IERC1155Royalty {
    /**
     * @dev Set the URI for a given token ID.
     *
     * @param tokenId the ID of the token for which to set the URI.
     * @param newTokenURI the new URI for the token.
     */
    function setTokenURI(uint256 tokenId, string memory newTokenURI) external;

    /**
     * @dev Retrieve the URI for a given token ID.
     *
     * @param tokenId the ID of the token for which to retrieve the URI.
     *
     * @return the URI of the token.
     */
    function uri(uint256 tokenId) external view returns (string memory);

    /**
     * @dev Mint a new ERC-1155 token with the specified royalty and token URI.
     *
     * @param account the address that will receive the newly minted token.
     * @param id the ID for the token to be minted.
     * @param amount the amount of tokens to mint.
     * @param royalty the royalty rate for the token.
     * @param royaltyReceiver the address that will receive the royalty.
     * @param tokenURI the URI for the token.
     */
    function mint(
        address account,
        uint256 id,
        uint256 amount,
        uint256 royalty,
        address royaltyReceiver,
        string memory tokenURI
    ) external;

    /**
     * @dev Retrieve royalty information for a given token ID and sale price.
     *
     * @param tokenId the ID of the token for which to retrieve royalty information.
     * @param salePrice the sale price of the token.
     *
     * @return receiver the address that should receive the royalty.
     * @return royaltyAmount the amount of the royalty.
     */
    function royaltyInfo(
        uint256 tokenId,
        uint256 salePrice
    ) external view returns (address receiver, uint256 royaltyAmount);

     /**
     * @dev Retrieve the balance of tokens held by an address.
     *
     * @param account the address of the token holder.
     * @param id the token ID to check the balance of.
     *
     * @return the balance of the specified token ID for the account.
     */
    function balanceOf(address account, uint256 id) external view returns (uint256);
}
