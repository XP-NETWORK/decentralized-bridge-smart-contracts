// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/token/common/ERC2981.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/IHederaTokenService.sol";

contract RoyaltyInfoProxy is HederaTokenService {
    function royaltyInfo(
        address token,
        int64 serialNumber
    ) external returns (int256, IHederaTokenService.NonFungibleTokenInfo memory) {
        (
            int256 response,
            IHederaTokenService.NonFungibleTokenInfo memory nft
        ) = getNonFungibleTokenInfo(token, serialNumber);
        return (response, nft);
    }
}
