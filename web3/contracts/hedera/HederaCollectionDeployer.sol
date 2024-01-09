// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./HederaNFT.sol";
import "./HederaNFTClaim.sol";
import "./interfaces/INFTCollectionDeployer.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/IHederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaResponseCodes.sol";
import "../../lib/hedera/contracts/hts-precompile/ExpiryHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/KeyHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/FeeHelper.sol";

contract HederaCollectionDeployer is
    INFTCollectionDeployer,
    HederaTokenService,
    ExpiryHelper,
    FeeHelper,
    KeyHelper
{
    address public owner;
    int64 public constant DEFAULT_EXPIRY = 7890000;
    int64 public constant MAX_INT = 0xFFFFFFFF;

    constructor() {}

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call");
        _;
    }

    event CreatedCollection(address collectionAddress);

    function setOwner(address _owner) external {
        require(owner == address(0), "Owner already set!");
        owner = _owner;
    }

    function deployNFT721Collection(
        string memory name,
        string memory symbol,
        int64 royaltyNum,
        address royaltyReceiver
    ) external override onlyOwner returns (address) {
        IHederaTokenService.TokenKey[]
            memory keys = new IHederaTokenService.TokenKey[](1);
        keys[0] = getSingleKey(
            KeyHelper.KeyType.SUPPLY,
            KeyHelper.KeyValueType.CONTRACT_ID,
            owner
        );

        IHederaTokenService.HederaToken memory token;
        token.name = name;
        token.symbol = symbol;
        token.treasury = owner;
        token.memo = "";
        token.tokenSupplyType = true;
        token.maxSupply = MAX_INT;
        token.freezeDefault = false;
        token.tokenKeys = keys;
        token.expiry = createAutoRenewExpiry(owner, DEFAULT_EXPIRY);
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

        emit CreatedCollection(createdToken);

        return createdToken;
    }
}
