// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "hardhat/console.sol";

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./interfaces/INFTStorageERC721.sol";
import "./interfaces/INFTStorageERC1155.sol";
import "./interfaces/IERC721Royalty.sol";
import "./interfaces/IERC1155Royalty.sol";
import "./interfaces/INFTStorageDeployer.sol";
import "./interfaces/INFTCollectionDeployer.sol";
import "./AddressUtilityLib.sol";
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

contract Bridge {
    using ECDSA for bytes32;
    using AddressUtilityLib for string;

    mapping(address => Validator) public validators;
    mapping(bytes32 => bool) public uniqueIdentifier;

    mapping(address => bool) public blackListedValidators;

    INFTCollectionDeployer public collectionDeployer;
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
    // collectionAddress => source NftStorage1155
    mapping(string => mapping(string => address))
        public originalStorageMapping1155;

    // collectionAddress => source NftStorage721
    mapping(string => mapping(string => address))
        public duplicateStorageMapping721;
    // collectionAddress => source NftStorage1155
    mapping(string => mapping(string => address))
        public duplicateStorageMapping1155;

    mapping(address validator => bool signed) private uniqueValidators;

    string public selfChain = "";
    string constant TYPEERC721 = "singular"; // a more general term to accomodate non-evm chains
    string constant TYPEERC1155 = "multiple"; // a more general term to accomodate non-evm chains

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
        string lockTxChain; // The tx on which the nft is locked.
    }

    event AddNewValidator(address _validator);
    event BlackListValidator(address _validator);
    event RewardValidator(address _validator);

    event Locked(
        uint256 tokenId, // Unique ID for the NFT transfer
        string destinationChain, // Chain to where the NFT is being transferred
        string destinationUserAddress, // User's address in the destination chain
        string sourceNftContractAddress, // Address of the NFT contract in the source chain
        uint256 tokenAmount, // Token amount is 1 incase it is ERC721
        string nftType, // NFT type is either 721 or 1155.
        string sourceChain, // Name of the chain emitting
        string metaDataUri // Metadata uri of nft 
    );

    event UnLock721(address to, uint256 tokenId, address contractAddr);

    event UnLock1155(
        address to,
        uint256 tokenId,
        address contractAddr,
        uint256 amount
    );

    event Claimed721(
        string lockTxChain, // Chain on which the lock txn hapenned.
        string sourceChain, // Chain from where the NFT is being transferred
        string transactionHash, // Transaction hash of the transfer on the source chain
        address nftContract, // The contract address from which tokens were claimed
        uint256 tokenId  // The tokenId that was claimed
    );

    event Claim1155(
        string lockTxChain, // Chain on which the lock txn hapenned.
        string sourceChain, // Chain from where the NFT is being transferred
        string transactionHash, // Transaction hash of the transfer on the source chain
        address nftContract, // The contract address from which tokens were claimed
        uint256 tokenId,  // The tokenId that was claimed
        uint256 amount // The amount of the tokens that were claimed
    );

    modifier requireFees() {
        require(msg.value > 0, "Tx Fees is required!");
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
        address _collectionDeployer,
        address _storageDeployer,
        address _collectionOwner
    ) {
        require(
            _collectionDeployer != address(0),
            "Address cannot be zero address!"
        );
        require(
            _storageDeployer != address(0),
            "Address cannot be zero address!"
        );

        collectionDeployer = INFTCollectionDeployer(_collectionDeployer);
        storageDeployer = INFTStorageDeployer(_storageDeployer);

        collectionDeployer.setOwner(_collectionOwner, address(this));
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
        require(!blackListedValidators[_validator], "validator blacklisted");
        require(_validator != address(0), "Address cannot be zero address!");
        require(signatures.length > 0, "Must have signatures!");
        require(!validators[_validator].added, "Validator already added");

        bytes[] memory signaturesArr = new bytes[](signatures.length);

        for (uint256 i = 0; i < signatures.length; i++) {
            signaturesArr[i] = signatures[i].signature;
        }

        verifySignature(keccak256(abi.encode(_validator)), signaturesArr);

        emit AddNewValidator(address(_validator));
        validators[_validator].added = true;
        validatorsCount += 1;
    }

    function blackListValidator(
        address _validator,
        SignerAndSignature[] memory signatures
    ) external {
        require(_validator != address(0), "Address cannot be zero address!");
        require(signatures.length > 0, "Must have signatures!");
        require(validators[_validator].added, "Validator is not added");

        bytes[] memory signaturesArr = new bytes[](signatures.length);

        for (uint256 i = 0; i < signatures.length; i++) {
            signaturesArr[i] = signatures[i].signature;
        }

        verifySignature(keccak256(abi.encode(_validator,"blackList")), signaturesArr);

        emit BlackListValidator(address(_validator));
        validators[_validator].added = false;
        validatorsCount -= 1;
        blackListedValidators[_validator] = true;
    }

    function claimValidatorRewards(
        address _validator
    ) external {
        require(_validator != address(0), "Address cannot be zero address!");
        require(
            validators[_validator].added == true,
            "Validator does not exist!"
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
        address sourceNftContractAddress, // Address of the NFT contract in the source chain
        string memory metaDataUri
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
                tokenId,
                destinationChain,
                destinationUserAddress,
                addressToString(sourceNftContractAddress),
                1,
                TYPEERC721,
                selfChain,
                metaDataUri
            );
        } else {
            transferToStorage721(
                duplicateStorageMapping721,
                sourceNftContractAddress,
                tokenId
            );
            // if duplicate, emit original address
            emit Locked(
                tokenId,
                destinationChain,
                destinationUserAddress,
                originalCollectionAddress.contractAddress,
                1,
                TYPEERC721,
                originalCollectionAddress.chain,
                metaDataUri
            );
        }
    }

    function lock1155(
        uint256 tokenId, // Unique ID for the NFT transfer
        string memory destinationChain, // Chain to where the NFT is being transferred
        string memory destinationUserAddress, // User's address in the destination chain
        address sourceNftContractAddress, // Address of the NFT contract in the source chain
        uint256 tokenAmount,
        string memory metaDataUri
    ) external {
        require(
            sourceNftContractAddress != address(0),
            "sourceNftContractAddress cannot be zero address"
        );
        require(tokenAmount > 0, "token amount must be > than zero");
        // Check if sourceNftContractAddress is original or duplicate
        DuplicateToOriginalContractInfo
            memory originalCollectionAddress = duplicateToOriginalMapping[
                sourceNftContractAddress
            ][selfChain];

        bool isOriginal = originalCollectionAddress
            .contractAddress
            .compareStrings("");

        if (isOriginal) {
            transferToStorage1155(
                originalStorageMapping1155,
                sourceNftContractAddress,
                tokenId,
                tokenAmount
            );

            // if original i.e does not exist in duplicate mapping
            // emit the sourceNftContractAddress we got as argument
            emit Locked(
                tokenId,
                destinationChain,
                destinationUserAddress,
                addressToString(sourceNftContractAddress),
                tokenAmount,
                TYPEERC1155,
                selfChain,
                metaDataUri
            );
        } else {
            transferToStorage1155(
                duplicateStorageMapping1155,
                sourceNftContractAddress,
                tokenId,
                tokenAmount
            );

            // if duplicate, emit original address
            emit Locked(
                tokenId,
                destinationChain,
                destinationUserAddress,
                originalCollectionAddress.contractAddress,
                tokenAmount,
                TYPEERC1155,
                originalCollectionAddress.chain,
                metaDataUri
            );
        }
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
        if (hasDuplicate && hasStorage) {
            IERC721Royalty duplicateCollection = IERC721Royalty(
                duplicateCollectionAddress.contractAddress.stringToAddress()
            );
            address ownerOfToken;
            try duplicateCollection.ownerOf(data.tokenId) returns (
                address _ownerOfToken
            ) {
                ownerOfToken = _ownerOfToken;
            } catch {}

            if (ownerOfToken == storageContract) {
                unLock721(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract
                );
            } else {
                duplicateCollection.mint(
                    data.destinationUserAddress,
                    data.tokenId,
                    data.royalty,
                    data.royaltyReceiver,
                    data.metadata
                );
            }
            emit Claimed721(data.lockTxChain, data.sourceChain, data.transactionHash, address(duplicateCollection), data.tokenId);
        }
        // ===============================/ hasDuplicate && NOT hasStorage /=======================
        else if (hasDuplicate && !hasStorage) {
            IERC721Royalty nft721Collection = IERC721Royalty(
                duplicateCollectionAddress.contractAddress.stringToAddress()
            );
            nft721Collection.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );

        emit Claimed721(data.lockTxChain, data.sourceChain, data.transactionHash, address(nft721Collection), data.tokenId);
        }
        // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
        else if (!hasDuplicate && !hasStorage) {
            IERC721Royalty newCollectionAddress = IERC721Royalty(
                collectionDeployer.deployNFT721Collection(
                    data.name,
                    data.symbol
                )
            );
            

            // update duplicate mappings
            originalToDuplicateMapping[data.sourceNftContractAddress][
                data.sourceChain
            ] = OriginalToDuplicateContractInfo(
                selfChain,
                addressToString(address(newCollectionAddress))
            );

            duplicateToOriginalMapping[address(newCollectionAddress)][
                selfChain
            ] = DuplicateToOriginalContractInfo(
                data.sourceChain,
                data.sourceNftContractAddress
            );

            newCollectionAddress.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );

        emit Claimed721(data.lockTxChain, data.sourceChain, data.transactionHash, address(newCollectionAddress), data.tokenId);
            // ===============================/ NOT hasDuplicate && hasStorage /=======================
        } else if (!hasDuplicate && hasStorage) {
            IERC721Royalty originalCollection = IERC721Royalty(
                data.sourceNftContractAddress.stringToAddress()
            );

            if (originalCollection.ownerOf(data.tokenId) == storageContract) {
                unLock721(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract
                );
            } else {
                // console.log("HERE2");

                // ============= This could be wrong. Need verification ============
                originalCollection.mint(
                    data.destinationUserAddress,
                    data.tokenId,
                    data.royalty,
                    data.royaltyReceiver,
                    data.metadata
                );
            }

        emit Claimed721(data.lockTxChain, data.sourceChain, data.transactionHash, address(originalCollection), data.tokenId);
            // ============= This could be wrong. Need verification ============
        } else {
            // TODO: remove after testing
            require(false, "Invalid bridge state");
        }

    }

    function claimNFT1155(
        ClaimData memory data,
        bytes[] memory signatures
    )
        external
        payable
        hasCorrectFee(data.fee)
        matchesCurrentChain(data.destinationChain)
    {
        require(
            keccak256(abi.encodePacked(data.nftType)) ==
                keccak256(abi.encodePacked(TYPEERC1155)),
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
            storageContract = duplicateStorageMapping1155[
                duplicateCollectionAddress.contractAddress
            ][selfChain];
        } else {
            storageContract = originalStorageMapping1155[
                data.sourceNftContractAddress
            ][data.sourceChain];
        }

        bool hasStorage = storageContract != address(0);

        // ===============================/ Is Duplicate && Has Storage /=======================
        if (hasDuplicate && hasStorage) {
            IERC1155Royalty collecAddress = IERC1155Royalty(
                duplicateCollectionAddress.contractAddress.stringToAddress()
            );
            uint256 balanceOfTokens = collecAddress.balanceOf(
                storageContract,
                data.tokenId
            );
            if (balanceOfTokens >= data.tokenAmount) {
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.tokenAmount
                );
            } else {
                uint256 toMint = data.tokenAmount - balanceOfTokens;
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    balanceOfTokens
                );
                collecAddress.mint(
                    data.destinationUserAddress,
                    data.tokenId,
                    toMint,
                    data.royalty,
                    data.royaltyReceiver,
                    data.metadata
                );
            }
            emit Claim1155(data.lockTxChain, data.sourceChain, data.transactionHash, address(collecAddress), data.tokenId, data.tokenAmount);
        }
        // ===============================/ Is Duplicate && No Storage /=======================
        else if (hasDuplicate && !hasStorage) {
            IERC1155Royalty nft1155Collection = IERC1155Royalty(
                duplicateCollectionAddress.contractAddress.stringToAddress()
            );
            nft1155Collection.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.tokenAmount,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );
             emit Claim1155(data.lockTxChain, data.sourceChain, data.transactionHash, address(nft1155Collection), data.tokenId, data.tokenAmount);
            
        }
        // ===============================/ Not Duplicate && No Storage /=======================
        else if (!hasDuplicate && !hasStorage) {
            IERC1155Royalty newCollectionAddress = IERC1155Royalty(
                collectionDeployer.deployNFT1155Collection()
            );
            //  new ERC1155Royalty();

            // update duplicate mappings
            originalToDuplicateMapping[data.sourceNftContractAddress][
                data.sourceChain
            ] = OriginalToDuplicateContractInfo(
                selfChain,
                addressToString(address(newCollectionAddress))
            );
            duplicateToOriginalMapping[address(newCollectionAddress)][
                selfChain
            ] = DuplicateToOriginalContractInfo(
                data.sourceChain,
                data.sourceNftContractAddress
            );

            newCollectionAddress.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.tokenAmount,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );
                emit Claim1155(data.lockTxChain, data.sourceChain, data.transactionHash, address(newCollectionAddress), data.tokenId, data.tokenAmount);
            // ===============================/ Duplicate && No Storage /=======================
        } else if (!hasDuplicate && hasStorage) {
            IERC1155Royalty collecAddress = IERC1155Royalty(
                data.sourceNftContractAddress.stringToAddress()
            );

            uint256 balanceOfTokens = collecAddress.balanceOf(
                storageContract,
                data.tokenId
            );

            if (balanceOfTokens >= data.tokenAmount) {
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.tokenAmount
                );
            } else {
                uint256 toMint = data.tokenAmount - balanceOfTokens;
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    balanceOfTokens
                );
                collecAddress.mint(
                    data.destinationUserAddress,
                    data.tokenId,
                    toMint,
                    data.royalty,
                    data.royaltyReceiver,
                    data.metadata
                );
            }
            emit Claim1155(data.lockTxChain, data.sourceChain, data.transactionHash, address(collecAddress), data.tokenId, data.tokenAmount);
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

    function unLock1155(
        address to,
        uint256 tokenId,
        address contractAddress,
        uint256 amountOfTokens
    ) private {
        // address nftStorageAddress1155 = originalStorageMapping1155[
        //     address(contractAddress)
        // ][sourceChain];

        // require(
        //     nftStorageAddress1155 != address(0),
        //     "NFT Storage contract does not exist!"
        // );

        // if storage contract exists in mapping, unlock token on the
        // storage contract
        INFTStorageERC1155 nftStorageContract = INFTStorageERC1155(
            contractAddress
        );

        emit UnLock1155(to, tokenId, address(contractAddress), amountOfTokens);

        nftStorageContract.unlockToken(tokenId, amountOfTokens, to);
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

        IERC721(sourceNftContractAddress).safeTransferFrom(
            msg.sender,
            storageAddress,
            tokenId
        );
    }

    function transferToStorage1155(
        mapping(string => mapping(string => address))
            storage storageMapping1155,
        address sourceNftContractAddress,
        uint256 tokenId,
        uint256 tokenAmount
    ) private {
        address storageAddress = storageMapping1155[
            addressToString(sourceNftContractAddress)
        ][selfChain];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = storageDeployer.deployNFT1155Storage(
                sourceNftContractAddress
            );
            // console.log("here %s", storageAddress);

            storageMapping1155[addressToString(sourceNftContractAddress)][
                selfChain
            ] = storageAddress;

            // address st = storageMapping1155[sourceNftContractAddress][
            //     selfChain
            // ];
            // console.log("here st %s", st);
        }

        IERC1155(sourceNftContractAddress).safeTransferFrom(
            msg.sender,
            storageAddress,
            tokenId,
            tokenAmount,
            ""
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
    ) private returns (address[] memory) {
        uint256 percentage = 0;
        address[] memory validatorsToReward = new address[](signatures.length);

        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recover(hash, signatures[i]);
            // console.log("signer: %s", signer);

            if (validators[signer].added && uniqueValidators[signer] == false) {
                percentage += 1;
                validatorsToReward[i] = signer;
                uniqueValidators[signer] = true;
            }
        }
        // Cleanup the mapping
        for (uint256 i = 0; i < validatorsToReward.length; i++) {
            delete uniqueValidators[validatorsToReward[i]];
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
                data.fee,
                data.lockTxChain
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
