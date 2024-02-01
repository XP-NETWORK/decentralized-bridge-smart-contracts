// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./interfaces/IERC721Royalty.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/IHederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaResponseCodes.sol";
import "../../lib/hedera/contracts/hts-precompile/ExpiryHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/KeyHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/FeeHelper.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "./interfaces/IHRC.sol";
import "./interfaces/INFTClaim.sol";

contract HederaNftDep is
    Ownable,
    HederaTokenService,
    IERC721Royalty,
    ExpiryHelper,
    FeeHelper,
    KeyHelper
{
    using EnumerableSet for EnumerableSet.UintSet;
    using Strings for uint256;

    int64 public constant DEFAULT_EXPIRY = 7890000;
    int64 public constant MAX_INT = 0xFFFFFFFF;
    address public htsToken;
    INFTClaim public nftClaim;
    event Transfer(
        address indexed from,
        address indexed to,
        int64 indexed tokenId
    );
    event Minted(address indexed htsToken, int64 indexed tokenId);

    mapping(address => bool) private associations;

    mapping(address => mapping(address => EnumerableSet.UintSet))
        private nftClaims;

    constructor(
        string memory name,
        string memory symbol,
        address owner,
        INFTClaim claimContract,
        int64 royaltyNum,
        address royaltyReceiver
    ) payable Ownable(owner) {
        require(royaltyNum <= 10000, "Royalty Numerator should be <= 10000");
        IHederaTokenService.TokenKey[]
            memory keys = new IHederaTokenService.TokenKey[](1);
        keys[0] = getSingleKey(
            KeyHelper.KeyType.SUPPLY,
            KeyHelper.KeyValueType.CONTRACT_ID,
            address(this)
        );
        nftClaim = claimContract;

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
        IHederaTokenService.RoyaltyFee[]
            memory royaltyFees = new IHederaTokenService.RoyaltyFee[](1);
        royaltyFees[0] = createRoyaltyFeeWithoutFallback(
            royaltyNum,
            10000,
            royaltyReceiver
        );

        IHederaTokenService.FixedFee[]
            memory fixedFees = new IHederaTokenService.FixedFee[](0);

        (int256 resp, address createdToken) = HederaTokenService
            .createNonFungibleTokenWithCustomFees(
                token,
                fixedFees,
                royaltyFees
            );

        require(resp == HederaResponseCodes.SUCCESS, "Failed to create token.");

        resp = int256(IHRC(createdToken).associate());
        require(
            resp == HederaResponseCodes.SUCCESS ||
                resp == HederaResponseCodes.TOKEN_ALREADY_ASSOCIATED_TO_ACCOUNT,
            "Failed to associate token"
        );

        htsToken = createdToken;
    }

    function _transferClaim(
        address to,
        int64 serialNum,
        address token
    ) private returns (int256) {
        int256 resp = transferNFT(token, address(this), to, serialNum);
        return resp;
    }

    function decodeHts(uint256 data) public pure returns (address, int64) {
        bytes32 d2 = bytes32(data);
        address token = address(uint160(bytes20(d2)));
        int96 serialNum = int96(uint96(uint256(d2)));

        return (token, int64(serialNum));
    }

    function safeTransferFrom(
        address _from,
        address _to,
        uint256 _serialNum
    ) external onlyOwner {
        (address token, int64 serial) = decodeHts(_serialNum);

        int256 resp;
        if (!associations[token]) {
            resp = int256(IHRC(token).associate());

            require(
                resp == HederaResponseCodes.SUCCESS ||
                    resp ==
                    HederaResponseCodes.TOKEN_ALREADY_ASSOCIATED_TO_ACCOUNT,
                "Failed to associate token."
            );
            associations[token] = true;
        }

        if (_to == owner()) {
            resp = transferNFT(token, _from, address(this), serial);
            emit Transfer(_from, address(this), serial);
        } else if (_from == owner()) {
            resp = transferNFT(token, address(this), _to, serial);
            emit Transfer(address(this), _to, serial);
            if (resp == HederaResponseCodes.TOKEN_NOT_ASSOCIATED_TO_ACCOUNT) {
                nftClaim.addClaimRecord(_to, token, serial);
            }
        }
        require(
            resp == HederaResponseCodes.SUCCESS,
            "Failed to transfer token."
        );
    }

    function royaltyInfo(
        uint256 tokenId,
        uint256 salePrice
    ) external override returns (address receiver, uint256 royaltyAmount) {
        (address token, int64 serialNumber) = decodeHts(tokenId);
        (
            int256 response,
            IHederaTokenService.NonFungibleTokenInfo memory nft
        ) = getNonFungibleTokenInfo(token, serialNumber);
        require(
            response == HederaResponseCodes.SUCCESS,
            "Failed to get token info"
        );
        if (nft.tokenInfo.royaltyFees.length == 0) {
            return (address(0), 0);
        }
        if (nft.tokenInfo.royaltyFees[0].amount == 0) {
            return (
                nft.tokenInfo.royaltyFees[0].feeCollector,
                (salePrice *
                    uint256(int256(nft.tokenInfo.royaltyFees[0].numerator))) /
                    uint256(int256(nft.tokenInfo.royaltyFees[0].denominator))
            );
        }
        return (
            nft.tokenInfo.royaltyFees[0].feeCollector,
            uint256(int256(nft.tokenInfo.royaltyFees[0].amount))
        );
    }

    function mint(
        address to,
        uint256,
        uint256,
        address,
        string memory tokenURI
    ) external override onlyOwner {
        bytes[] memory metadata = new bytes[](1);
        metadata[0] = abi.encodePacked(tokenURI);
        (int256 resp, , int64[] memory serialNum) = mintToken(
            htsToken,
            0,
            metadata
        );
        emit Minted(htsToken, serialNum[0]);
        require(resp == HederaResponseCodes.SUCCESS, "Failed to mint token. ");

        int256 tresp = _transferClaim(to, serialNum[0], htsToken);
        if (tresp == HederaResponseCodes.TOKEN_NOT_ASSOCIATED_TO_ACCOUNT) {
            nftClaim.addClaimRecord(to, htsToken, serialNum[0]);
            return;
        }

        require(
            tresp == HederaResponseCodes.SUCCESS,
            "Failed to transfer token."
        );
        emit Transfer(address(this), to, serialNum[0]);
    }

    function claimNft(int64 serialNum, address token) external {
        nftClaim.getClaimRecord(msg.sender, token, serialNum);
        int256 resp = _transferClaim(msg.sender, serialNum, token);
        require(
            resp == HederaResponseCodes.SUCCESS,
            "Failed to transfer token"
        );
        emit Transfer(address(this), msg.sender, serialNum);
    }

    function ownerOf(uint256 tokenId) external override returns (address) {
        (address token, int64 serialNumber) = decodeHts(tokenId);
        (
            int256 response,
            IHederaTokenService.NonFungibleTokenInfo memory tokenInfo
        ) = getNonFungibleTokenInfo(token, serialNumber);
        require(
            response == HederaResponseCodes.SUCCESS,
            "Failed to get token info"
        );
        return tokenInfo.ownerId;
    }
}
