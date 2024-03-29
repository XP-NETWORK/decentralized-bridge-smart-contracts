// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "hardhat/console.sol";

// import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./interfaces/INFTStorageERC721.sol";
import "./interfaces/INFTStorageDeployer.sol";
import "../AddressUtilityLib.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/IHederaTokenService.sol";
import "../../lib/hedera/contracts/hts-precompile/HederaResponseCodes.sol";
import "../../lib/hedera/contracts/hts-precompile/ExpiryHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/KeyHelper.sol";
import "../../lib/hedera/contracts/hts-precompile/FeeHelper.sol";
import "./interfaces/IHTSCompatabilityLayer.sol";
import "./BiDirectionalTokenInfoMapLib.sol";
/**
 * @dev Stucture to store signature with signer public address
 */
struct SignerAndSignature {
    string signerAddress;
    bytes signature;
}

struct DuplicateToOriginalContractInfo {
    string chain;
    string contractAddress;
}

struct OriginalToDuplicateContractInfo {
    string chain;
    string contractAddress;
}

struct Validator {
    bool added;
    uint pendingReward;
}

contract HederaBridge is
    HederaTokenService,
    ExpiryHelper,
    FeeHelper,
    KeyHelper
{
    using ECDSA for bytes32;
    using AddressUtilityLib for string;
    using BiDirectionalTokenInfoMapLib for TokenInfo;
    int64 public constant DEFAULT_EXPIRY = 7890000;
    int64 public constant MAX_INT = 0xFFFFFFFF;
    mapping(address => Validator) public validators;
    mapping(bytes32 => bool) public uniqueIdentifier;

    mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
        public keyToValue;
    mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
        public valueToKey;

    INFTStorageDeployer public storageDeployer;

    uint256 public validatorsCount = 0;

    // address[] validatorsArray;

    // originalCollectionAddress => destinationCollectionAddress
    mapping(string => mapping(string => OriginalToDuplicateContractInfo))
        public originalToDuplicateMapping;

    // destinationCollectionAddress => originalCollectionAddress
    mapping(address => mapping(string => DuplicateToOriginalContractInfo))
        public duplicateToOriginalMapping;

    // collectionAddress => source NftStorage721
    mapping(string => mapping(string => address))
        public originalStorageMapping721;

    // collectionAddress => source NftStorage721
    mapping(string => mapping(string => address))
        public duplicateStorageMapping721;

    string public selfChain = "";
    string constant TYPEERC721 = "singular"; // a more general term to accomodate non-evm chains

    struct ClaimData {
        uint256 tokenId; // Unique ID for the NFT transfer
        string sourceChain; // Chain from where the NFT is being transferred
        string destinationChain; // Chain to where the NFT is being transferred
        address destinationUserAddress; // User's address in the destination chain
        string sourceNftContractAddress; // Address of the NFT contract in the source chain
        string name; // name of NFT collection
        string symbol; // symbol of nft collection
        uint256 royalty; // royalty of nft collection
        address royaltyReceiver; // address of user who is going to receive royalty
        string metadata; // Metadata related to the NFT being transferred
        string transactionHash; // Transaction hash of the transfer on the source chain
        uint256 tokenAmount; // Number of NFTs being transferred
        string nftType; // Type of the NFT (could be ERC721 or ERC1155)
        uint256 fee; // fee that needs to be paid by the user to the bridge,
    }

    event AddNewValidator(address _validator);
    event RewardValidator(address _validator);

    event Locked(
        uint256 tokenId, // Unique ID for the NFT transfer
        string destinationChain, // Chain to where the NFT is being transferred
        string destinationUserAddress, // User's address in the destination chain
        string sourceNftContractAddress, // Address of the NFT contract in the source chain
        uint256 tokenAmount, // Token amount is 1 incase it is ERC721
        string nftType, // NFT type is either 721 or 1155.
        string sourceChain // Name of the chain emitting
    );

    event UnLock721(address to, uint256 tokenId, address contractAddr);

    event Claimed(
        string sourceChain, // Chain from where the NFT is being transferred
        string transactionHash, // Transaction hash of the transfer on the source chain
        address nftContract,
        uint256 emittedTokenId
    );

    modifier requireFees() {
        require(msg.value > 0, "Tx Fees is required!");
        _;
    }

    modifier onlyValidator() {
        require(false, "Failed to verify signature!");
        _;
    }

    modifier matchesCurrentChain(string memory destinationChain) {
        require(
            keccak256(abi.encodePacked(destinationChain)) ==
                keccak256(abi.encodePacked(selfChain)),
            "Invalid destination chain!"
        );
        _;
    }

    modifier hasCorrectFee(uint256 fee) {
        require(msg.value >= fee, "data.fee LESS THAN sent amount!");
        _;
    }

    constructor(
        address[] memory _validators,
        string memory _chainType,
        address _storageDeployer
    ) {
        require(
            _storageDeployer != address(0),
            "Address cannot be zero address!"
        );

        storageDeployer = INFTStorageDeployer(_storageDeployer);

        storageDeployer.setOwner(address(this));

        selfChain = _chainType;
        for (uint256 i = 0; i < _validators.length; i++) {
            validators[_validators[i]].added = true;
            validatorsCount += 1;
        }
    }

    function addValidator(
        address _validator,
        SignerAndSignature[] memory signatures
    ) external {
        require(_validator != address(0), "Address cannot be zero address!");
        require(signatures.length > 0, "Must have signatures!");
        require(!validators[_validator].added, "Validator already added");

        uint256 percentage = 0;
        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recover(
                keccak256(abi.encode(_validator)),
                signatures[i].signature
            );
            if (validators[signer].added) {
                percentage += 1;
            }
        }

        require(
            percentage >= ((validatorsCount * 2) / 3) + 1,
            "Threshold not reached!"
        );

        emit AddNewValidator(address(_validator));
        validators[_validator].added = true;
        validatorsCount += 1;
    }

    function claimValidatorRewards(
        address _validator,
        bytes[] memory signatures
    ) external {
        require(_validator != address(0), "Address cannot be zero address!");
        require(signatures.length > 0, "Must have signatures!");
        require(
            validators[_validator].added == true,
            "Validator does not exist!"
        );

        uint256 percentage = 0;
        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recover(
                keccak256(abi.encode(_validator)),
                signatures[i]
            );
            if (validators[signer].added) {
                percentage += 1;
            }
        }

        require(
            percentage >= ((validatorsCount * 2) / 3) + 1,
            "Threshold not reached!"
        );

        emit RewardValidator(address(_validator));
        uint256 rewards = validators[_validator].pendingReward;
        validators[_validator].pendingReward = 0;
        payable(_validator).transfer(rewards);
    }

    function lock721(
        uint256 tokenId, // Unique ID for the NFT transfer
        string memory destinationChain, // Chain to where the NFT is being transferred
        string memory destinationUserAddress, // User's address in the destination chain
        address sourceNftContractAddress // Address of the NFT contract in the source chain
    ) external {
        require(
            sourceNftContractAddress != address(0),
            "sourceNftContractAddress cannot be zero address"
        );
        // Check if sourceNftContractAddress is original or duplicate
        DuplicateToOriginalContractInfo
            memory originalCollectionAddress = duplicateToOriginalMapping[
                sourceNftContractAddress
            ][selfChain];

        bool isOriginal = originalCollectionAddress
            .contractAddress
            .compareStrings("");

        TokenInfo memory tinfo;

        if (
            BiDirectionalTokenInfoMapLib.containsValue(
                TokenInfo(
                    tokenId,
                    selfChain,
                    addressToString(sourceNftContractAddress),
                    true
                ),
                keyToValue,
                valueToKey
            )
        ) {
            tinfo = BiDirectionalTokenInfoMapLib.getId(
                TokenInfo(
                    tokenId,
                    selfChain,
                    addressToString(sourceNftContractAddress),
                    true
                ),
                keyToValue,
                valueToKey
            );
        }

        uint256 mutid = 0;

        if (tinfo.exists) {
            mutid = tinfo.tokenId;
        } else {
            mutid = tokenId;
            BiDirectionalTokenInfoMapLib.insert(
                TokenInfo(
                    tokenId,
                    selfChain,
                    addressToString(sourceNftContractAddress),
                    true
                ),
                TokenInfo(
                    tokenId,
                    selfChain,
                    addressToString(sourceNftContractAddress),
                    true
                ),
                keyToValue,
                valueToKey
            );
        }

        // isOriginal
        if (isOriginal) {
            transferToStorage721(
                originalStorageMapping721,
                sourceNftContractAddress,
                tokenId
            );

            // if original i.e does not exist in duplicate mapping
            // emit the sourceNftContractAddress we got as argument
            emit Locked(
                mutid,
                destinationChain,
                destinationUserAddress,
                addressToString(sourceNftContractAddress),
                1,
                TYPEERC721,
                selfChain
            );
        } else {
            transferToStorage721(
                duplicateStorageMapping721,
                sourceNftContractAddress,
                tokenId
            );
            // if duplicate, emit original address
            emit Locked(
                mutid,
                destinationChain,
                destinationUserAddress,
                originalCollectionAddress.contractAddress,
                1,
                TYPEERC721,
                originalCollectionAddress.chain
            );
        }
    }

    function deployCollection(
        string memory name,
        string memory symbol,
        int64 royaltyNum,
        address royaltyReceiver
    ) private returns (address) {
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
        IHederaTokenService.RoyaltyFee[] memory royaltyFees;
        if (royaltyNum > 0) {
            royaltyFees = new IHederaTokenService.RoyaltyFee[](1);
            royaltyFees[0] = createRoyaltyFeeWithoutFallback(
                royaltyNum,
                10000,
                royaltyReceiver
            );
        } else {
            royaltyFees = new IHederaTokenService.RoyaltyFee[](0);
        }
        IHederaTokenService.FixedFee[]
            memory fixedFees = new IHederaTokenService.FixedFee[](0);

        (
            int256 resp,
            address createdToken
        ) = createNonFungibleTokenWithCustomFees(token, fixedFees, royaltyFees);

        require(resp == HederaResponseCodes.SUCCESS, "Failed to create token.");
        return createdToken;
    }

    function mintHtsNft(
        address ctr,
        string memory tokenURI,
        address to,
        uint256 tokenId,
        string memory srcChain,
        string memory sourceNftContractAddr
    ) private returns (address, uint256) {
        bytes[] memory metadata = new bytes[](1);
        metadata[0] = abi.encodePacked(tokenURI);
        (int256 resp, , int64[] memory serialNum) = mintToken(ctr, 0, metadata);
        require(resp == HederaResponseCodes.SUCCESS, "Failed to mint token. ");
        BiDirectionalTokenInfoMapLib.insert(
            TokenInfo(tokenId, srcChain, sourceNftContractAddr, true),
            TokenInfo(
                uint256(uint64(serialNum[0])),
                selfChain,
                addressToString(ctr),
                true
            ),
            keyToValue,
            valueToKey
        );

        int256 tresp = transferNFT(ctr, address(this), to, serialNum[0]);
        require(
            tresp == HederaResponseCodes.SUCCESS,
            "Failed to transfer minted token."
        );
        return (ctr, uint256(uint64(serialNum[0])));
    }

    function claimNFT721(
        ClaimData memory data,
        bytes[] memory signatures
    )
        external
        payable
        hasCorrectFee(data.fee)
        matchesCurrentChain(data.destinationChain)
    {
        // console.log("msg.value %s", msg.value);
        require(
            keccak256(abi.encodePacked(data.nftType)) ==
                keccak256(abi.encodePacked(TYPEERC721)),
            "Invalid NFT type!"
        );
        bytes32 hash = createClaimDataHash(data);

        require(!uniqueIdentifier[hash], "Data already processed!");
        uniqueIdentifier[hash] = true;

        address[] memory validatorsToReward = verifySignature(hash, signatures);
        rewardValidators(data.fee, validatorsToReward);

        OriginalToDuplicateContractInfo
            memory duplicateCollectionAddress = originalToDuplicateMapping[
                data.sourceNftContractAddress
            ][data.sourceChain];

        bool hasDuplicate = !duplicateCollectionAddress
            .contractAddress
            .compareStrings("");

        address storageContract;
        if (hasDuplicate) {
            storageContract = duplicateStorageMapping721[
                duplicateCollectionAddress.contractAddress
            ][selfChain];
        } else {
            storageContract = originalStorageMapping721[
                data.sourceNftContractAddress
            ][data.sourceChain];
        }

        bool hasStorage = storageContract != address(0);

        // console.log("STORAGE CONTRACT: %s", storageContract);
        // console.log(
        //     "DUPLICATE ADDRESS: %s",
        //     duplicateCollectionAddress.contractAddress
        // );
        // console.log("HAS STORAGE: %s", hasStorage);
        // console.log("HAS duplicate: %s", hasDuplicate);

        // ===============================/ hasDuplicate && hasStorage /=======================

        TokenInfo memory tinfo;
        if (
            BiDirectionalTokenInfoMapLib.containsId(
                TokenInfo(
                    data.tokenId,
                    data.sourceChain,
                    data.sourceNftContractAddress,
                    true
                ),
                keyToValue,
                valueToKey
            )
        ) {
            tinfo = BiDirectionalTokenInfoMapLib.getValue(
                TokenInfo(
                    data.tokenId,
                    data.sourceChain,
                    data.sourceNftContractAddress,
                    true
                ),
                keyToValue,
                valueToKey
            );
        }

        if (hasDuplicate && hasStorage) {
            IHTSCompatibilityLayer htscl = IHTSCompatibilityLayer(
                duplicateCollectionAddress.contractAddress.stringToAddress()
            );

            address ownerOfToken;
            if (tinfo.exists) {
                try htscl.ownerOf(tinfo.tokenId) returns (
                    address _ownerOfToken
                ) {
                    ownerOfToken = _ownerOfToken;
                } catch {}
            } else {
                try htscl.ownerOf(data.tokenId) returns (
                    address _ownerOfToken
                ) {
                    ownerOfToken = _ownerOfToken;
                } catch {}
            }

            if (ownerOfToken == storageContract && tinfo.exists) {
                unLock721(
                    data.destinationUserAddress,
                    tinfo.tokenId,
                    storageContract
                );
                emit Claimed(data.sourceChain, data.transactionHash, duplicateCollectionAddress.contractAddress.stringToAddress(), tinfo.tokenId);
            } else {
                     (address nftContract, uint256 tokenId ) = mintHtsNft(
                    duplicateCollectionAddress
                        .contractAddress
                        .stringToAddress(),
                    data.metadata,
                    data.destinationUserAddress,
                    data.tokenId,
                    data.sourceChain,
                    data.sourceNftContractAddress
                );
                emit Claimed(data.sourceChain, data.transactionHash, nftContract, tokenId);
            }

        }
        // ===============================/ hasDuplicate && NOT hasStorage /=======================
        else if (hasDuplicate && !hasStorage) {
              (address nftContract, uint256 tokenId ) = mintHtsNft(
                duplicateCollectionAddress.contractAddress.stringToAddress(),
                data.metadata,
                data.destinationUserAddress,
                data.tokenId,
                data.sourceChain,
                data.sourceNftContractAddress
            );
            emit Claimed(data.sourceChain, data.transactionHash, nftContract, tokenId);
        }
        // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
        else if (!hasDuplicate && !hasStorage) {
            address newCollectionAddress = deployCollection(
                data.name,
                data.symbol,
                int64(int256(data.royalty)),
                data.royaltyReceiver
            );
            // update duplicate mappings
            originalToDuplicateMapping[data.sourceNftContractAddress][
                data.sourceChain
            ] = OriginalToDuplicateContractInfo(
                selfChain,
                addressToString(newCollectionAddress)
            );

            duplicateToOriginalMapping[newCollectionAddress][
                selfChain
            ] = DuplicateToOriginalContractInfo(
                data.sourceChain,
                data.sourceNftContractAddress
            );

          (address nftContract, uint256 tokenId ) = mintHtsNft(
                newCollectionAddress,
                data.metadata,
                data.destinationUserAddress,
                data.tokenId,
                data.sourceChain,
                data.sourceNftContractAddress
            );

        
        emit Claimed(data.sourceChain, data.transactionHash, nftContract, tokenId);
            // ===============================/ NOT hasDuplicate && hasStorage /=======================
        } else if (!hasDuplicate && hasStorage) {
            IHTSCompatibilityLayer htscl = IHTSCompatibilityLayer(
                data.sourceNftContractAddress.stringToAddress()
            );
            address ownerOfToken;
            if (tinfo.exists) {
                try htscl.ownerOf(tinfo.tokenId) returns (
                    address _ownerOfToken
                ) {
                    ownerOfToken = _ownerOfToken;
                } catch {}
            } else {
                try htscl.ownerOf(data.tokenId) returns (
                    address _ownerOfToken
                ) {
                    ownerOfToken = _ownerOfToken;
                } catch {}
            }

            if (ownerOfToken == storageContract && tinfo.exists) {
                unLock721(
                    data.destinationUserAddress,
                    tinfo.tokenId,
                    storageContract
                );
            } else {
                (address nftContract, uint256 tokenId ) = mintHtsNft(
                    data.sourceNftContractAddress.stringToAddress(),
                    data.metadata,
                    data.destinationUserAddress,
                    data.tokenId,
                    data.sourceChain,
                    data.sourceNftContractAddress
                );
        emit Claimed(data.sourceChain, data.transactionHash, nftContract, tokenId);
            }
        } else {
            // TODO: remove after testing
            require(false, "Invalid bridge state");
        }

    }

    function unLock721(
        address to,
        uint256 tokenId,
        address contractAddress
    ) private {
        // address nftStorageAddress721 = originalStorageMapping721[
        //     address(contractAddress)
        // ][sourceChain];
        // console.log("HERE 3 %s", nftStorageAddress721);
        // require(
        //     nftStorageAddress721 != address(0),
        //     "NFT Storage contract does not exist!"
        // );

        // if storage contract exists in mapping, unlock token on the
        // storage contract
        INFTStorageERC721 nftStorageContract = INFTStorageERC721(
            contractAddress
        );

        emit UnLock721(to, tokenId, address(contractAddress));

        nftStorageContract.unlockToken(tokenId, to);
    }

    function transferToStorage721(
        mapping(string => mapping(string => address)) storage storageMapping721,
        address sourceNftContractAddress,
        uint256 tokenId
    ) private {
        address storageAddress = storageMapping721[
            addressToString(sourceNftContractAddress)
        ][selfChain];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = storageDeployer.deployNFT721Storage(
                sourceNftContractAddress
            );

            storageMapping721[addressToString(sourceNftContractAddress)][
                selfChain
            ] = storageAddress;
        }

        transferNFT(
            sourceNftContractAddress,
            msg.sender,
            storageAddress,
            int64(uint64(tokenId))
        );
    }

    function rewardValidators(
        uint256 fee,
        address[] memory validatorsToReward
    ) private {
        require(fee > 0, "Invalid fees");

        uint256 totalRewards = address(this).balance;
        // console.log("totalRewards %s", totalRewards);

        require(totalRewards >= fee, "No rewards available");

        uint256 feePerValidator = totalRewards / validatorsToReward.length;
        // console.log("FEE %s", feePerValidator);
        for (uint256 i = 0; i < validatorsToReward.length; i++) {
            validators[validatorsToReward[i]].pendingReward += feePerValidator;
            // payable().transfer(feePerValidator);
        }
    }

    function verifySignature(
        bytes32 hash,
        bytes[] memory signatures
    ) private view returns (address[] memory) {
        uint256 percentage = 0;
        address[] memory validatorsToReward = new address[](signatures.length);
        // console.log("validator1: %s", validatorsArray[0]);
        // console.log("validator2: %s", validatorsArray[1]);

        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recover(hash, signatures[i]);
            // console.log("signer: %s", signer);

            if (validators[signer].added) {
                percentage += 1;
                validatorsToReward[i] = signer;
            }
        }
        // emit LogHash(hash, signatures);
        // console.log("Percentage: %s", percentage);
        //  console.log("Percentage: %s", msg.value);
        require(
            percentage >= ((validatorsCount * 2) / 3) + 1,
            "Threshold not reached!"
        );

        return validatorsToReward;
    }

    event LogHash(bytes32 indexed hashValue, bytes[]);

    function createClaimDataHash(
        ClaimData memory data
    ) private pure returns (bytes32) {
        // console.log("data.tokenId %s", data.tokenId);
        // console.log("data.sourceChain %s", data.sourceChain);
        // console.log("data.destinationChain %s", data.destinationChain);
        // console.log(
        //     "data.destinationUserAddress %s",
        //     data.destinationUserAddress
        // );
        // console.log(
        //     "data.sourceNftContractAddress %s",
        //     data.sourceNftContractAddress
        // );
        // console.log("data.name %s", data.name);
        // console.log("data.symbol %s", data.symbol);
        // console.log("data.royalty %s", data.royalty);
        // console.log("data.royaltyReceiver %s", data.royaltyReceiver);
        // console.log("data.metadata %s", data.metadata);
        // console.log("data.transactionHash %s", data.transactionHash);
        // console.log("data.tokenAmount %s", data.tokenAmount);
        // console.log("data.nftType %s", data.nftType);
        // console.log("data.fee %s", data.fee);

        bytes32 hash = keccak256(
            abi.encode(
                data.tokenId,
                data.sourceChain,
                data.destinationChain,
                data.destinationUserAddress,
                data.sourceNftContractAddress,
                data.name,
                data.symbol,
                data.royalty,
                data.royaltyReceiver,
                data.metadata,
                data.transactionHash,
                data.tokenAmount,
                data.nftType,
                data.fee
            )
        );

        return hash;
    }

    function recover(
        bytes32 hash,
        bytes memory sig
    ) private pure returns (address) {
        hash = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", hash)
        );
        return ECDSA.recover(hash, sig);
    }

    /**
     * @dev Converts an Ethereum address to its string representation.
     * @param _address The Ethereum address to convert.
     * @return The string representation of the Ethereum address.
     */
    function addressToString(
        address _address
    ) internal pure returns (string memory) {
        bytes32 value = bytes32(uint256(uint160(_address))); // Corrected casting
        bytes memory alphabet = "0123456789abcdef";

        bytes memory str = new bytes(42);
        str[0] = 0x30; // '0'
        str[1] = 0x78; // 'x'
        for (uint256 i = 0; i < 20; i++) {
            str[2 + i * 2] = alphabet[uint8(value[i + 12] >> 4)];
            str[3 + i * 2] = alphabet[uint8(value[i + 12] & 0x0f)];
        }
        return string(str);
    }

    receive() external payable {}
}
