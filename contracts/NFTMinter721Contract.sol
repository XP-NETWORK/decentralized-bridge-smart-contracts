// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract NFTMinter721Contract is ERC721, Ownable {
    string private _baseTokenURI;

    constructor(
        string memory collectionName,
        string memory symbol,
        string memory baseTokenURI
    ) ERC721(collectionName, symbol)   {
        _baseTokenURI = baseTokenURI;
    }

    function _baseURI() internal view override returns (string memory) {
        return _baseTokenURI;
    }

    function setBaseURI(string memory baseTokenURI) external onlyOwner {
        _baseTokenURI = baseTokenURI;
    }

    function mint(address recipient, uint256 tokenId) public onlyOwner {
        _safeMint(recipient, tokenId);
    }
}
