// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/token/common/ERC2981.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/IHederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/IHederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaResponseCodes.sol";
import "../../lib/hedera/contracts/hts-precompile/ExpiryHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/KeyHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/FeeHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/IHRC.sol";

contract ContractProxy is
    HederaTokenService,
    ExpiryHelper,
    KeyHelper
{
    int64 public constant MAX_INT = 0xFFFFFFFF;
    int64 public constant DEFAULT_EXPIRY = 7890000;

    event NftCollectionCreated(address token);
    event Mint(address token, address owner, int64 serialNumber);

    function royaltyInfo(
        address token,
        int64 serialNumber
    )
        external
        returns (int256, IHederaTokenService.NonFungibleTokenInfo memory)
    {
        (
            int256 response,
            IHederaTokenService.NonFungibleTokenInfo memory nft
        ) = getNonFungibleTokenInfo(token, serialNumber);
        return (response, nft);
    }

    function deployNft(
        string memory name,
        string memory symbol
    ) public payable {
        IHederaTokenService.TokenKey[]
            memory keys = new IHederaTokenService.TokenKey[](1);
        keys[0] = getSingleKey(
            KeyHelper.KeyType.SUPPLY,
            KeyHelper.KeyValueType.CONTRACT_ID,
            address(this)
        );

        IHederaTokenService.HederaToken memory token;
        token.name = name;
        token.symbol = symbol;
        token.treasury = address(this);
        token.memo = "";
        token.tokenSupplyType = true;
        token.maxSupply = MAX_INT;
        token.freezeDefault = false;
        token.tokenKeys = keys;
        token.expiry = createAutoRenewExpiry(address(this), DEFAULT_EXPIRY);

        (int256 resp, address createdToken) = HederaTokenService
            .createNonFungibleToken(
                token
            );
        require(
            resp == HederaResponseCodes.SUCCESS,
            string(
                abi.encodePacked(
                    "Failed to create token. Error Code: ",
                    Strings.toString(uint256(resp))
                )
            )
        );

        resp = int256(IHRC(createdToken).associate());
        require(
            resp == HederaResponseCodes.SUCCESS ||
                resp == HederaResponseCodes.TOKEN_ALREADY_ASSOCIATED_TO_ACCOUNT,
            "Failed to associate token"
        );
        emit NftCollectionCreated(createdToken);
    }

    function mint(address token, string memory tokenURI) public payable {
        uint256 uAssocResp = IHRC(token).associate();
        int256 assocResp = int256(uAssocResp);
        require(
            assocResp == HederaResponseCodes.SUCCESS ||
                assocResp ==
                HederaResponseCodes.TOKEN_ALREADY_ASSOCIATED_TO_ACCOUNT,
            string(
                abi.encodePacked(
                    "Failed to associate token. Error Code: ",
                    Strings.toString(uAssocResp)
                )
            )
        );
        bytes[] memory metadata = new bytes[](1);
        metadata[0] = abi.encodePacked(tokenURI);
        (int256 resp, , int64[] memory serialNum) = HederaTokenService.mintToken(
            token,
            0,
            metadata
        );
        require(
            resp == HederaResponseCodes.SUCCESS,
            string(
                abi.encodePacked(
                    "Failed to mint token. Error Code: ",
                    Strings.toString(uint256(resp))
                )
            )
        );
        resp = HederaTokenService.transferNFT(token, address(this), msg.sender, serialNum[0]);
        require(
            resp == HederaResponseCodes.SUCCESS,
            string(
                abi.encodePacked(
                    "Failed to transfer token. ",
                    Strings.toString(uint256(resp))
                )
            )
        );
        emit Mint(token, msg.sender, serialNum[0]);
    }
}