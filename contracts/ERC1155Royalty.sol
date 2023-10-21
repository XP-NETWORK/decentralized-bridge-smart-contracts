// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/common/ERC2981.sol";

contract ERC1155Royalty is ERC1155, Ownable, IERC2981 {
    mapping(uint256 => address) private _royaltyReceivers;
    mapping(uint256 => uint256) private _royalties;
    mapping(uint256 => string) private _tokenURIs;

    constructor(address owner) ERC1155("") Ownable(owner) {}

    function setTokenURI(
        uint256 tokenId,
        string memory newTokenURI
    ) public onlyOwner {
        _tokenURIs[tokenId] = newTokenURI;
    }

    function uri(uint256 tokenId) public view override returns (string memory) {
        // If there's a specific URI set for this token, use that. Otherwise, use the base URI.
        return _tokenURIs[tokenId];
    }

    function mint(
        address account,
        uint256 id,
        uint256 amount,
        uint256 royalty,
        address royaltyReceiver,
        string memory tokenURI
    ) public onlyOwner {
        require(royalty <= 10000, "Royalty too high");

        _mint(account, id, amount, "");
        _royaltyReceivers[id] = royaltyReceiver;
        _royalties[id] = royalty;
        _tokenURIs[id] = tokenURI;
    }

    function royaltyInfo(
        uint256 tokenId,
        uint256 salePrice
    ) external view override returns (address receiver, uint256 royaltyAmount) {
        receiver = _royaltyReceivers[tokenId];
        royaltyAmount = (salePrice * _royalties[tokenId]) / 10000;
    }
}
