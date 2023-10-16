// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract TestNFTerc721 is ERC721, Ownable {
    // Counter for token IDs
    uint256 private _tokenIdCounter;

    // Base URI for metadata
    string private _baseTokenURI;

    constructor(string memory name, string memory symbol, string memory baseURI) ERC721(name, symbol) {
        _baseTokenURI = baseURI;
    }

    // Mint a new NFT
    function mint(address to) public onlyOwner {
        uint256 tokenId = _tokenIdCounter;
        _mint(to, tokenId);
        _tokenIdCounter++;
    }

    // Set the base token URI
    function setBaseTokenURI(string memory baseURI) public onlyOwner {
        _baseTokenURI = baseURI;
    }

    // Override the base URI function to include the token ID
    function _baseURI() internal view override returns (string memory) {
        return _baseTokenURI;
    }
}
