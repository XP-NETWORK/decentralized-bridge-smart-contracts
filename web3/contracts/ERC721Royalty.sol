// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/token/common/ERC2981.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ERC721Royalty is ERC721URIStorage, Ownable, IERC2981 {
    mapping(uint256 => address) private _royalityRecievers;
    mapping(uint256 => uint256) private _royalties;

    address public bridge;

    constructor(
        string memory name,
        string memory symbol,
        address owner
    ) ERC721(name, symbol) Ownable(owner) {}

    modifier onlyBridge() {
        require(msg.sender == bridge, "Not bridge!");
        _;
    }

    function mint(
        address to,
        uint256 tokenId,
        uint256 royalty,
        address royalityReciever,
        string memory tokenURI
    ) external onlyBridge {
        require(royalty <= 10000, "Royalty too high"); // royalty is up to 10000 representing 100.00%
        _mint(to, tokenId);
        _setTokenURI(tokenId, tokenURI); // set the individual token URI
        _royalityRecievers[tokenId] = royalityReciever;
        _royalties[tokenId] = royalty;
    }

    function royaltyInfo(
        uint256 tokenId,
        uint256 salePrice
    ) external view override returns (address receiver, uint256 royaltyAmount) {
        receiver = _royalityRecievers[tokenId];
        royaltyAmount = _royalties[tokenId];
    }
}
