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
import "./NFTMinter721Contract.sol";
import "./NFTMinter1155Contract.sol";

contract Bridge {
    using ECDSA for bytes32;

    mapping(address => bool) public validators;
    mapping(bytes32 => bool) public uniqueIdentifier;

    uint256 validatorsCount = 0;

    // originalCollectionAddress => destinationCollectionAddress
    mapping(address => address) public originalToDuplicateMapping;

    // destinationCollectionAddress => originalCollectionAddress
    mapping(address => address) public duplicateToOriginalMapping;

    // collectionAddress => source NftStorage721
    mapping(address => address) public originalStorageMapping721;
    // collectionAddress => source NftStorage1155
    mapping(address => address) public originalStorageMapping1155;

    // collectionAddress => source NftStorage721
    mapping(address => address) public duplicateStorageMapping721;
    // collectionAddress => source NftStorage1155
    mapping(address => address) public duplicateStorageMapping1155;

    address[] public addressList;
    uint256 private txFees = 0x0;
    string public chainSymbol = "";
    string constant TYPEERC721 = "singular"; // a more general term to accomodate non-evm chains
    string constant TYPEERC1155 = "multiple"; // a more general term to accomodate non-evm chains

    struct ClaimData1155 {
        uint256 tokenId; // Unique ID for the NFT transfer
        string destinationChain; // Chain to where the NFT is being transferred
        address destinationUserAddress; // User's address in the destination chain
        address sourceNftContractAddress; // Address of the NFT contract in the source chain
        string nftType; // Type of the NFT (could be ERC721 or ERC1155)
        uint256 tokenAmount; // Number of NFTs being transferred
        string sourceChain; // Chain from where the NFT is being transferred
        string name; // name of NFT collection
        string symbol; // symbol of nft collection
        uint256 royality; // royality of nft collection
        address royalityReciever; // address of user who is going to receive royalty
        string metadata; // Metadata related to the NFT being transferred
        string transactionHash; // Transaction hash of the transfer on the source chain
        uint256 fee; // fee that needs to be paid by the user to the bridge
        bytes[] signatures;
    }

    struct ClaimData721 {
        uint256 tokenId; // Unique ID for the NFT transfer
        string destinationChain; // Chain to where the NFT is being transferred
        address destinationUserAddress; // User's address in the destination chain
        address sourceNftContractAddress; // Address of the NFT contract in the source chain
        string nftType; // Type of the NFT (could be ERC721 or ERC1155)
        string sourceChain; // Chain from where the NFT is being transferred
        string name; // name of NFT collection
        string symbol; // symbol of nft collection
        uint256 royality; // royality of nft collection
        address royalityReciever; // address of user who is going to receive royalty
        string metadata; // Metadata related to the NFT being transferred
        string transactionHash; // Transaction hash of the transfer on the source chain
        uint256 fee; // fee that needs to be paid by the user to the bridge,
        bytes[] signatures;
    }

    event AddNewValidator(address _validator);

    event Locked(
        uint256 tokenId, // Unique ID for the NFT transfer
        string destinationChain, // Chain to where the NFT is being transferred
        string destinationUserAddress, // User's address in the destination chain
        address sourceNftContractAddress, // Address of the NFT contract in the source chain
        uint256 tokenAmount, // Token amount is 1 incase it is ERC721
        string nftType // NFT type is either 721 or 1155.
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
                keccak256(abi.encodePacked(chainSymbol)),
            "Invalid destination chain"
        );
        _;
    }

    modifier hasCorrectFee(uint256 fee) {
        require(fee == msg.value, "Fee and sent amount do not match");
        _;
    }

    constructor(address[] memory _validators, string memory _chainSymbol) {
        chainSymbol = _chainSymbol;
        for (uint256 i = 0; i < _validators.length; i++) {
            validators[_validators[i]] = true;
            addressList.push(_validators[i]);
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
        if (percentage >= (addressList.length * 2) / 3) {
            emit AddNewValidator(address(_validator));
            validators[_validator] = true;
            addressList.push(_validator);
            validatorsCount += 1;
        }
    }

    function transferToStorage721(
        mapping(address => address) storage storageMapping721,
        address sourceNftContractAddress,
        uint256 tokenId
    ) internal {
        address storageAddress = storageMapping721[
            address(sourceNftContractAddress)
        ];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = address(
                new NftStorageERC721(address(sourceNftContractAddress))
            );
            storageMapping721[
                address(sourceNftContractAddress)
            ] = storageAddress;
        }

        IERC721(sourceNftContractAddress).safeTransferFrom(
            msg.sender,
            address(storageAddress),
            tokenId
        );
    }

    function transferToStorage1155(
        mapping(address => address) storage storageMapping1155,
        address sourceNftContractAddress,
        uint256 tokenId,
        uint256 tokenAmount
    ) internal {
        address storageAddress = storageMapping1155[
            address(sourceNftContractAddress)
        ];

        // NOT hasStorage
        if (storageAddress == address(0)) {
            storageAddress = address(
                new NftStorageERC1155(address(sourceNftContractAddress))
            );
            storageMapping1155[
                address(sourceNftContractAddress)
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
        address originalCollectionAddress = duplicateToOriginalMapping[
            sourceNftContractAddress
        ];

        // isOriginal
        if (originalCollectionAddress == address(0)) {
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
                TYPEERC721
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
                address(originalCollectionAddress),
                1,
                TYPEERC721
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
        address originalCollectionAddress = duplicateToOriginalMapping[
            address(sourceNftContractAddress)
        ];

        bool isOriginal = originalCollectionAddress == address(0);

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
                TYPEERC1155
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
                address(originalCollectionAddress),
                tokenAmount,
                TYPEERC1155
            );
        }
    }

    function claimNFT721(
        ClaimData721 memory data
    )
        external
        payable
        hasCorrectFee(data.fee)
        matchesCurrentChain(data.destinationChain)
    {
        bytes32 hash = claimNFT721Hash(data);

        require(!uniqueIdentifier[hash], "Data already processed!");
        uniqueIdentifier[hash] = true;

        verifySignature(hash, data.signatures);

        address sourceNftContractAddress = data.sourceNftContractAddress;
        address duplicateCollectionAddress = originalToDuplicateMapping[
            sourceNftContractAddress
        ];

        bool hasDuplicate = duplicateCollectionAddress != address(0);

        address storageContract;
        if (hasDuplicate) {
            storageContract = duplicateStorageMapping721[
                sourceNftContractAddress
            ];
        } else {
            storageContract = originalStorageMapping721[
                sourceNftContractAddress
            ];
        }

        bool hasStorage = storageContract != address(0);

        // ===============================/ hasDuplicate && hasStorage /=======================
        if (hasDuplicate && hasStorage) {
            NFTMinter721Contract duplicateCollection = NFTMinter721Contract(
                duplicateCollectionAddress
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
                    data.tokenId
                );
            }
        }
        // ===============================/ hasDuplicate && NOT hasStorage /=======================
        else if (hasDuplicate && !hasStorage) {
            NFTMinter721Contract nft721Collection = NFTMinter721Contract(
                duplicateCollectionAddress
            );
            nft721Collection.mint(data.destinationUserAddress, data.tokenId);
        }
        // ===============================/ NOT hasDuplicate && NOT hasStorage /=======================
        else if (!hasDuplicate && !hasStorage) {
            NFTMinter721Contract newCollectionAddress = new NFTMinter721Contract(
                    data.name,
                    data.symbol,
                    data.metadata
                );

            // update duplicate mappings
            originalToDuplicateMapping[sourceNftContractAddress] = address(
                newCollectionAddress
            );
            duplicateToOriginalMapping[
                address(newCollectionAddress)
            ] = sourceNftContractAddress;
            newCollectionAddress.mint(
                data.destinationUserAddress,
                data.tokenId
            );
            // ===============================/ NOT hasDuplicate && hasStorage /=======================
        } else if (!hasDuplicate && hasStorage) {
            NFTMinter721Contract originalCollection = NFTMinter721Contract(
                sourceNftContractAddress
            );

            if (originalCollection.ownerOf(data.tokenId) == storageContract) {
                unLock721(
                    data.destinationUserAddress,
                    data.tokenId,
                    storageContract
                );
            } else {
                // ============= This could be wrong. Need verification ============
                originalCollection.mint(
                    data.destinationUserAddress,
                    data.tokenId
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
        ClaimData1155 memory data
    )
        external
        payable
        hasCorrectFee(data.fee)
        matchesCurrentChain(data.destinationChain)
    {
        bytes32 hash = claimNFT1155Hash(data);

        require(!uniqueIdentifier[hash], "Data already processed!");
        uniqueIdentifier[hash] = true;

        verifySignature(hash, data.signatures);

        address sourceNftContractAddress = data.sourceNftContractAddress;
        address duplicateCollectionAddress = originalToDuplicateMapping[
            sourceNftContractAddress
        ];

        bool hasDuplicate = duplicateCollectionAddress != address(0);

        address storageContract;
        if (hasDuplicate) {
            storageContract = duplicateStorageMapping1155[
                sourceNftContractAddress
            ];
        } else {
            storageContract = originalStorageMapping1155[
                sourceNftContractAddress
            ];
        }

        bool hasStorage = storageContract != address(0);

        // ===============================/ Is Duplicate && Has Storage /=======================
        if (hasDuplicate && hasStorage) {
            NFTMinter1155Contract collecAddress = NFTMinter1155Contract(
                duplicateCollectionAddress
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
                    ""
                );
            }
        }
        // ===============================/ Is Duplicate && No Storage /=======================
        else if (hasDuplicate && !hasStorage) {
            NFTMinter1155Contract nft1155Collection = NFTMinter1155Contract(
                duplicateCollectionAddress
            );
            nft1155Collection.mint(
                msg.sender,
                data.tokenId,
                data.tokenAmount,
                ""
            );
        }
        // ===============================/ Not Duplicate && No Storage /=======================
        else if (!hasDuplicate && !hasStorage) {
            NFTMinter1155Contract newCollectionAddress = new NFTMinter1155Contract(
                    data.metadata
                );

            // update duplicate mappings
            originalToDuplicateMapping[sourceNftContractAddress] = address(
                newCollectionAddress
            );
            duplicateToOriginalMapping[
                address(newCollectionAddress)
            ] = sourceNftContractAddress;
            newCollectionAddress.mint(
                msg.sender,
                data.tokenId,
                data.tokenAmount,
                ""
            );
            // ===============================/ Duplicate && No Storage /=======================
        } else if (!hasDuplicate && hasStorage) {
            NFTMinter1155Contract collecAddress = NFTMinter1155Contract(
                sourceNftContractAddress
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
                    ""
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
    ) internal {
        address nftStorageAddress721 = originalStorageMapping721[
            address(contractAddress)
        ];

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
        uint256 amountOfTokens
    ) internal {
        address nftStorageAddress1155 = originalStorageMapping1155[
            address(contractAddress)
        ];

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

    function verifySignature(bytes32 hash, bytes[] memory signatures) internal {
        uint256 percentage = 0;

        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recover(hash, signatures[i]);

            if (validators[signer]) {
                percentage += 1;
            }
        }

        require(
            percentage >= (addressList.length * 2) / 3,
            "Threshold not reached!"
        );
    }

    function claimNFT721Hash(
        ClaimData721 memory data
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    data.tokenId,
                    data.destinationChain,
                    data.destinationUserAddress,
                    data.sourceNftContractAddress,
                    data.nftType,
                    data.sourceChain,
                    data.name,
                    data.symbol,
                    data.royality,
                    data.royalityReciever,
                    data.metadata,
                    data.transactionHash,
                    data.fee
                )
            );
    }

    function claimNFT1155Hash(
        ClaimData1155 memory data
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    data.tokenId,
                    data.destinationChain,
                    data.destinationUserAddress,
                    data.sourceNftContractAddress,
                    data.nftType,
                    data.tokenAmount,
                    data.sourceChain,
                    data.name,
                    data.symbol,
                    data.royality,
                    data.royalityReciever,
                    data.metadata,
                    data.transactionHash,
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

    // function validatorSignature(bytes32 hash, bytes memory sig) external {
    //     require(validators[msg.sender], "Not a valid validator");
    //     require(!hasConfirmed(hash, msg.sender), "Signature already confirmed");

    //     address signer = ECDSA.recover(ECDSA.toEthSignedMessageHash(hash), sig);
    //     require(validators[signer], "Not a valid validator");

    //     bytes32 sigHash = keccak256(abi.encodePacked(hash, msg.sender));
    //     signatureCount[sigHash]++;

    //     require(signatureCount[sigHash] * 3 >= 11 * 2, "Not enough valid");
    //     if (signatureCount[sigHash] * 3 >= 11 * 2) {
    //         resetSignatureCount(hash, msg.sender);
    //     }
    // }

    // function hasConfirmed(
    //     bytes32 hash,
    //     address validator
    // ) internal view returns (bool) {
    //     bytes32 sigHash = keccak256(abi.encodePacked(hash, validator));
    //     return signatureCount[sigHash] > 0;
    // }

    // function resetSignatureCount(bytes32 hash, address validator) internal {
    //     bytes32 sigHash = keccak256(
    //         abi.encodePacked(hash, validators[msg.sender])
    //     );
    // }
}
