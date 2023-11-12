// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/token/common/ERC2981.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

interface IERC721Royalty {
    /**
     * @dev Mint a new ERC-721 token with the specified royalty and token URI.
     *
     * @param to the address that will receive the newly minted token.
     * @param tokenId the ID for the token to be minted.
     * @param royalty the royalty rate for the token.
     * @param royaltyReceiver the address that will receive the royalty.
     * @param tokenURI the URI for the token.
     */
    function mint(
        address to,
        uint256 tokenId,
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
     * @dev Retrieve the owner of the given token ID.
     *
     * @param tokenId the ID of the token.
     *
     * @return owner's address.
     */
    function ownerOf(uint256 tokenId) external view returns (address);
}
