// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "hardhat/console.sol";

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./NftStorageERC721.sol";
import "./NftStorageERC1155.sol";
import "./ERC721Royalty.sol";
import "./ERC1155Royalty.sol";

struct ContractInfo {
    string chain;
    address contractAddress;
}

contract Bridge {
    using ECDSA for bytes32;

    mapping(address => bool) public validators;
    mapping(bytes32 => bool) public uniqueIdentifier;

    uint256 validatorsCount = 0;

    // originalCollectionAddress => destinationCollectionAddress
    mapping(address => mapping(string => ContractInfo))
        public originalToDuplicateMapping;

    // destinationCollectionAddress => originalCollectionAddress
    mapping(address => mapping(string => ContractInfo))
        public duplicateToOriginalMapping;

    // collectionAddress => source NftStorage721
    mapping(address => mapping(string => address))
        public originalStorageMapping721;
    // collectionAddress => source NftStorage1155
    mapping(address => mapping(string => address))
        public originalStorageMapping1155;

    // collectionAddress => source NftStorage721
    mapping(address => mapping(string => address))
        public duplicateStorageMapping721;
    // collectionAddress => source NftStorage1155
    mapping(address => mapping(string => address))
        public duplicateStorageMapping1155;

    uint256 private txFees = 0x0;
    string public selfChain = "";
    string constant TYPEERC721 = "singular"; // a more general term to accomodate non-evm chains
    string constant TYPEERC1155 = "multiple"; // a more general term to accomodate non-evm chains

    struct ClaimData {
        uint256 tokenId; // Unique ID for the NFT transfer
        string sourceChain; // Chain from where the NFT is being transferred
        string destinationChain; // Chain to where the NFT is being transferred
        address destinationUserAddress; // User's address in the destination chain
        address sourceNftContractAddress; // Address of the NFT contract in the source chain
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

    event Locked(
        uint256 tokenId, // Unique ID for the NFT transfer
        string destinationChain, // Chain to where the NFT is being transferred
        string destinationUserAddress, // User's address in the destination chain
        address sourceNftContractAddress, // Address of the NFT contract in the source chain
        uint256 tokenAmount, // Token amount is 1 incase it is ERC721
        string nftType, // NFT type is either 721 or 1155.
        string chain // Name of the chain emitting
    );

    event UnLock721(address to, uint256 tokenId, address contractAddr);

    event UnLock1155(
        address to,
        uint256 tokenId,
        address contractAddr,
        uint256 amount
    );

    event Claimed(
        string sourceChain, // Chain from where the NFT is being transferred
        string transactionHash // Transaction hash of the transfer on the source chain
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
            "Invalid destination chain"
        );
        _;
    }

    modifier hasCorrectFee(uint256 fee) {
        require(fee == msg.value, "Fee and sent amount do not match");
        _;
    }

    constructor(address[] memory _validators, string memory _chainSymbol) {
        selfChain = _chainSymbol;
        for (uint256 i = 0; i < _validators.length; i++) {
            validators[_validators[i]] = true;
        }
    }

    function addValidator(address _validator, bytes[] memory sigs) public {
        uint256 percentage = 0;
        for (uint256 i = 0; i < sigs.length; i++) {
            address signer = recover(hashSwap(_validator), sigs[i]);
            // console.log("I am a console log!");
            if (validators[signer]) {
                // console.log("I am a console log2222!");
                percentage += 1;
            }
        }
        if (percentage >= (validatorsCount * 2) / 3) {
            emit AddNewValidator(address(_validator));
            validators[_validator] = true;
            validatorsCount += 1;
        }
    }

    function transferToStorage721(
        mapping(address => mapping(string => address))
            storage storageMapping721,
        address sourceNftContractAddress,
        uint256 tokenId
    ) internal {
        address storageAddress = storageMapping721[
            address(sourceNftContractAddress)
        ][selfChain];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = address(
                new NftStorageERC721(address(sourceNftContractAddress))
            );
            storageMapping721[address(sourceNftContractAddress)][
                selfChain
            ] = storageAddress;
        }

        IERC721(sourceNftContractAddress).safeTransferFrom(
            msg.sender,
            address(storageAddress),
            tokenId
        );
    }

    function transferToStorage1155(
        mapping(address => mapping(string => address))
            storage storageMapping1155,
        address sourceNftContractAddress,
        uint256 tokenId,
        uint256 tokenAmount
    ) internal {
        address storageAddress = storageMapping1155[
            address(sourceNftContractAddress)
        ][selfChain];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = address(
                new NftStorageERC1155(address(sourceNftContractAddress))
            );
            storageMapping1155[address(sourceNftContractAddress)][
                selfChain
            ] = storageAddress;
        }

        IERC1155(sourceNftContractAddress).safeTransferFrom(
            msg.sender,
            address(storageAddress),
            tokenId,
            tokenAmount,
            ""
        );
    }

    function lock721(
        uint256 tokenId, // Unique ID for the NFT transfer
        string memory destinationChain, // Chain to where the NFT is being transferred
        string memory destinationUserAddress, // User's address in the destination chain
        address sourceNftContractAddress // Address of the NFT contract in the source chain
    ) external {
        // Check if sourceNftContractAddress is original or duplicate
        ContractInfo
            memory originalCollectionAddress = duplicateToOriginalMapping[
                sourceNftContractAddress
            ][selfChain];

        // isOriginal
        if (originalCollectionAddress.contractAddress == address(0)) {
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
                address(sourceNftContractAddress),
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
                tokenId,
                destinationChain,
                destinationUserAddress,
                address(originalCollectionAddress.contractAddress),
                1,
                TYPEERC721,
                originalCollectionAddress.chain
            );
        }
    }

    function lock1155(
        uint256 tokenId, // Unique ID for the NFT transfer
        string memory destinationChain, // Chain to where the NFT is being transferred
        string memory destinationUserAddress, // User's address in the destination chain
        address sourceNftContractAddress, // Address of the NFT contract in the source chain
        uint256 tokenAmount
    ) external payable requireFees {
        // Check if sourceNftContractAddress is original or duplicate
        ContractInfo
            memory originalCollectionAddress = duplicateToOriginalMapping[
                address(sourceNftContractAddress)
            ][selfChain];

        bool isOriginal = originalCollectionAddress.contractAddress ==
            address(0);

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
                address(sourceNftContractAddress),
                tokenAmount,
                TYPEERC1155,
                selfChain
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
                address(originalCollectionAddress.contractAddress),
                tokenAmount,
                TYPEERC1155,
                originalCollectionAddress.chain
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
        bytes32 hash = createClaimDataHash(data);

        require(!uniqueIdentifier[hash], "Data already processed!");
        uniqueIdentifier[hash] = true;

        address[] memory validatorsToReward = verifySignature(hash, signatures);
        rewardValidators(data.fee, validatorsToReward);

        ContractInfo
            memory duplicateCollectionAddress = originalToDuplicateMapping[
                data.sourceNftContractAddress
            ][data.sourceChain];

        bool hasDuplicate = duplicateCollectionAddress.contractAddress !=
            address(0);

        address storageContract;
        if (hasDuplicate) {
            storageContract = duplicateStorageMapping721[
                data.sourceNftContractAddress
            ][data.sourceChain];
        } else {
            storageContract = originalStorageMapping721[
                data.sourceNftContractAddress
            ][data.sourceChain];
        }

        bool hasStorage = storageContract != address(0);

        // ===============================/ hasDuplicate && hasStorage /=======================
        if (hasDuplicate && hasStorage) {
            ERC721Royalty duplicateCollection = ERC721Royalty(
                duplicateCollectionAddress.contractAddress
            );

            if (duplicateCollection.ownerOf(data.tokenId) == storageContract) {
                unLock721(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.sourceChain
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
        }
        // ===============================/ hasDuplicate && NOT hasStorage /=======================
        else if (hasDuplicate && !hasStorage) {
            ERC721Royalty nft721Collection = ERC721Royalty(
                duplicateCollectionAddress.contractAddress
            );
            nft721Collection.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );
        }
        // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
        else if (!hasDuplicate && !hasStorage) {
            ERC721Royalty newCollectionAddress = new ERC721Royalty(
                data.name,
                data.symbol
            );

            // update duplicate mappings
            originalToDuplicateMapping[data.sourceNftContractAddress][
                data.sourceChain
            ] = ContractInfo(selfChain, address(newCollectionAddress));

            duplicateToOriginalMapping[address(newCollectionAddress)][
                selfChain
            ] = ContractInfo(data.sourceChain, data.sourceNftContractAddress);

            newCollectionAddress.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );
            // ===============================/ NOT hasDuplicate && hasStorage /=======================
        } else if (!hasDuplicate && hasStorage) {
            ERC721Royalty originalCollection = ERC721Royalty(
                data.sourceNftContractAddress
            );

            if (originalCollection.ownerOf(data.tokenId) == storageContract) {
                unLock721(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.sourceChain
                );
            } else {
                // ============= This could be wrong. Need verification ============
                originalCollection.mint(
                    data.destinationUserAddress,
                    data.tokenId,
                    data.royalty,
                    data.royaltyReceiver,
                    data.metadata
                );
            }
            // ============= This could be wrong. Need verification ============
        } else {
            // TODO: remove after testing
            require(false, "Invalid bridge state");
        }

        emit Claimed(data.sourceChain, data.transactionHash);
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
        bytes32 hash = createClaimDataHash(data);

        require(!uniqueIdentifier[hash], "Data already processed!");
        uniqueIdentifier[hash] = true;

        address[] memory validatorsToReward = verifySignature(hash, signatures);
        rewardValidators(data.fee, validatorsToReward);

        ContractInfo
            memory duplicateCollectionAddress = originalToDuplicateMapping[
                data.sourceNftContractAddress
            ][data.sourceChain];

        bool hasDuplicate = duplicateCollectionAddress.contractAddress !=
            address(0);

        address storageContract;
        if (hasDuplicate) {
            storageContract = duplicateStorageMapping1155[
                data.sourceNftContractAddress
            ][data.sourceChain];
        } else {
            storageContract = originalStorageMapping1155[
                data.sourceNftContractAddress
            ][data.sourceChain];
        }

        bool hasStorage = storageContract != address(0);

        // ===============================/ Is Duplicate && Has Storage /=======================
        if (hasDuplicate && hasStorage) {
            ERC1155Royalty collecAddress = ERC1155Royalty(
                duplicateCollectionAddress.contractAddress
            );
            if (collecAddress.balanceOf(storageContract, data.tokenId) > 0) {
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.tokenAmount,
                    data.sourceChain
                );
            } else {
                collecAddress.mint(
                    data.destinationUserAddress,
                    data.tokenId,
                    data.tokenAmount,
                    data.royalty,
                    data.royaltyReceiver,
                    data.metadata
                );
            }
        }
        // ===============================/ Is Duplicate && No Storage /=======================
        else if (hasDuplicate && !hasStorage) {
            ERC1155Royalty nft1155Collection = ERC1155Royalty(
                duplicateCollectionAddress.contractAddress
            );
            nft1155Collection.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.tokenAmount,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );
        }
        // ===============================/ Not Duplicate && No Storage /=======================
        else if (!hasDuplicate && !hasStorage) {
            ERC1155Royalty newCollectionAddress = new ERC1155Royalty();

            // update duplicate mappings
            originalToDuplicateMapping[data.sourceNftContractAddress][
                data.sourceChain
            ] = ContractInfo(selfChain, address(newCollectionAddress));
            duplicateToOriginalMapping[address(newCollectionAddress)][
                data.sourceChain
            ] = ContractInfo(data.sourceChain, data.sourceNftContractAddress);
            newCollectionAddress.mint(
                data.destinationUserAddress,
                data.tokenId,
                data.tokenAmount,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );
            // ===============================/ Duplicate && No Storage /=======================
        } else if (!hasDuplicate && hasStorage) {
            ERC1155Royalty collecAddress = ERC1155Royalty(
                data.sourceNftContractAddress
            );
            if (collecAddress.balanceOf(storageContract, data.tokenId) > 0) {
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.tokenAmount,
                    data.sourceChain
                );
            } else {
                collecAddress.mint(
                    data.destinationUserAddress,
                    data.tokenId,
                    data.tokenAmount,
                    data.royalty,
                    data.royaltyReceiver,
                    data.metadata
                );
            }
        } else {
            // TODO: remove after testing
            require(false, "Invalid bridge state");
        }

        emit Claimed(data.sourceChain, data.transactionHash);
    }

    function unLock721(
        address to,
        uint256 tokenId,
        address contractAddress,
        string memory sourceChain
    ) internal {
        address nftStorageAddress721 = originalStorageMapping721[
            address(contractAddress)
        ][sourceChain];

        require(
            nftStorageAddress721 != address(0),
            "NFT Storage contract does not exist!"
        );

        // if storage contract exists in mapping, unlock token on the
        // storage contract
        NftStorageERC721 nftStorageContract = NftStorageERC721(
            nftStorageAddress721
        );

        emit UnLock721(to, tokenId, address(contractAddress));

        nftStorageContract.unlockToken(tokenId);
    }

    function unLock1155(
        address to,
        uint256 tokenId,
        address contractAddress,
        uint256 amountOfTokens,
        string memory sourceChain
    ) internal {
        address nftStorageAddress1155 = originalStorageMapping1155[
            address(contractAddress)
        ][sourceChain];

        require(
            nftStorageAddress1155 != address(0),
            "NFT Storage contract does not exist!"
        );

        // if storage contract exists in mapping, unlock token on the
        // storage contract
        NftStorageERC1155 nftStorageContract = NftStorageERC1155(
            nftStorageAddress1155
        );

        emit UnLock1155(to, tokenId, address(contractAddress), amountOfTokens);

        nftStorageContract.unlockToken(tokenId, amountOfTokens);
    }

    function rewardValidators(
        uint256 fee,
        address[] memory validatorsToReward
    ) internal {
        require(fee > 0, "Invalid fees");

        uint256 totalRewards = address(this).balance;

        require(totalRewards >= fee, "No rewards available");

        uint256 feePerValidator = fee / validatorsToReward.length;

        for (uint256 i = 0; i < validatorsToReward.length; i++) {
            payable(validatorsToReward[i]).transfer(feePerValidator);
        }
    }

    function verifySignature(
        bytes32 hash,
        bytes[] memory signatures
    ) internal view returns (address[] memory) {
        uint256 percentage = 0;
        address[] memory validatorsToReward = new address[](signatures.length);

        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recover(hash, signatures[i]);

            if (validators[signer]) {
                percentage += 1;
                validatorsToReward[i] = signer;
            }
        }

        require(
            percentage >= (validatorsCount * 2) / 3,
            "Threshold not reached!"
        );

        return validatorsToReward;
    }

    function createClaimDataHash(
        ClaimData memory data
    ) internal pure returns (bytes32) {
        return
            keccak256(
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
    }

    function claimFee(address payable receiver) external {
        uint256 sendAmt = txFees;
        txFees = 0;
        receiver.transfer(sendAmt);
    }

    function hashSwap(address _validator) private pure returns (bytes32) {
        return keccak256(abi.encode(_validator));
    }

    function recover(
        bytes32 hash,
        bytes memory sig
    ) private pure returns (address) {
        return ECDSA.recover(hash, sig);
    }
}
