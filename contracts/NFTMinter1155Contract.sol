// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC1155/extensions/ERC1155.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract NFTMinter1155Contract is ERC1155, Ownable {
    string private _baseTokenURI;

    constructor(string memory baseTokenURI) ERC1155(baseTokenURI) {
        _baseTokenURI = baseTokenURI;
    }

    function uri(uint256 tokenId) public view override returns (string memory) {
        return string(abi.encodePacked(_baseTokenURI, tokenId));
    }

    function mint(address recipient, uint256 tokenId, uint256 amount, bytes memory data) public onlyOwner {
        _mint(recipient, tokenId, amount, data);
    }
}