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

struct ContractInfo {
    string chain;
    address contractAddress;
}

contract Bridge {
    using ECDSA for bytes32;

    mapping(address => bool) public validators;
    mapping(bytes32 => bool) public uniqueIdentifier;

    INFTCollectionDeployer public collectionDeployer;
    INFTStorageDeployer public storageDeployer;

    uint256 public validatorsCount = 0;

    // address[] validatorsArray;

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
        string sourceChain // Name of the chain emitting
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
        require(fee >= msg.value, "Fee and sent amount do not match");
        _;
    }

    constructor(
        address[] memory _validators,
        string memory _chainSymbol,
        address _collectionDeployer,
        address _storageDeployer
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

        collectionDeployer.setOwner(address(this));
        storageDeployer.setOwner(address(this));

        selfChain = _chainSymbol;
        for (uint256 i = 0; i < _validators.length; i++) {
            validators[_validators[i]] = true;
            validatorsCount += 1;
            // validatorsArray.push(_validators[i]);
        }
    }

    function addValidator(
        address _validator,
        bytes[] memory signatures
    ) external {
        require(_validator != address(0), "Address cannot be zero address");
        require(signatures.length > 0, "Must have signatures");

        uint256 percentage = 0;
        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recover(
                keccak256(abi.encode(_validator)),
                signatures[i]
            );
            if (validators[signer]) {
                percentage += 1;
            }
        }
        if (percentage >= ((validatorsCount * 2) / 3) + 1) {
            emit AddNewValidator(address(_validator));
            validators[_validator] = true;
            validatorsCount += 1;
        }
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

        bool isOriginal = originalCollectionAddress.contractAddress ==
            address(0);

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
    ) external {
        // Check if sourceNftContractAddress is original or duplicate
        ContractInfo
            memory originalCollectionAddress = duplicateToOriginalMapping[
                sourceNftContractAddress
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
                duplicateCollectionAddress.contractAddress
            );

            if (duplicateCollection.ownerOf(data.tokenId) == storageContract) {
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
        }
        // ===============================/ hasDuplicate && NOT hasStorage /=======================
        else if (hasDuplicate && !hasStorage) {
            IERC721Royalty nft721Collection = IERC721Royalty(
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
            IERC721Royalty newCollectionAddress = IERC721Royalty(
                collectionDeployer.deployNFT721Collection(
                    data.name,
                    data.symbol
                )
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
            IERC721Royalty originalCollection = IERC721Royalty(
                data.sourceNftContractAddress
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
                duplicateCollectionAddress.contractAddress
            );
            if (collecAddress.balanceOf(storageContract, data.tokenId) > 0) {
                // console.log("should come here");
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.tokenAmount
                );
            } else {
                // console.log("should NOT come here");
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
            IERC1155Royalty nft1155Collection = IERC1155Royalty(
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
            IERC1155Royalty newCollectionAddress = IERC1155Royalty(
                collectionDeployer.deployNFT1155Collection()
            );
            //  new ERC1155Royalty();

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
                data.tokenAmount,
                data.royalty,
                data.royaltyReceiver,
                data.metadata
            );
            // ===============================/ Duplicate && No Storage /=======================
        } else if (!hasDuplicate && hasStorage) {
            IERC1155Royalty collecAddress = IERC1155Royalty(
                data.sourceNftContractAddress
            );
            if (collecAddress.balanceOf(storageContract, data.tokenId) > 0) {
                unLock1155(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract,
                    data.tokenAmount
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
        mapping(address => mapping(string => address))
            storage storageMapping721,
        address sourceNftContractAddress,
        uint256 tokenId
    ) private {
        address storageAddress = storageMapping721[sourceNftContractAddress][
            selfChain
        ];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = storageDeployer.deployNFT721Storage(
                sourceNftContractAddress
            );

            storageMapping721[sourceNftContractAddress][
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
        mapping(address => mapping(string => address))
            storage storageMapping1155,
        address sourceNftContractAddress,
        uint256 tokenId,
        uint256 tokenAmount
    ) private {
        address storageAddress = storageMapping1155[sourceNftContractAddress][
            selfChain
        ];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = storageDeployer.deployNFT1155Storage(
                sourceNftContractAddress
            );
            // console.log("here %s", storageAddress);

            storageMapping1155[sourceNftContractAddress][
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

        require(totalRewards >= fee, "No rewards available");

        uint256 feePerValidator = fee / validatorsToReward.length;

        for (uint256 i = 0; i < validatorsToReward.length; i++) {
            payable(validatorsToReward[i]).transfer(feePerValidator);
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

            if (validators[signer]) {
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
}
