// SPDX-License-Identifier: MIT
pragma solidity >=0.8.19 <0.9.0;

import "hardhat/console.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/IERC721Metadata.sol";
import "@openzeppelin/contracts/token/ERC1155/extensions/IERC1155MetadataURI.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./NftStorageERC721.sol";
import "./NftStorageERC1155.sol";
import "./NFTMinter721Contract.sol";
import "./NFTMinter1155Contract.sol";

contract Bridge {
    using ECDSA for bytes32;

    mapping(address => bool) public validators;

    // originalCollectionAddress => destinationCollectionAddress
    mapping(address => address) public originalToDuplicateMapping;

    // destinationCollectionAddress => originalCollectionAddress
    mapping(address => address) public duplicateToOriginalMapping;

    // collectionAddress => source NftStorage721
    mapping(address => address) public storageMapping721;

    // collectionAddress => source NftStorage1155
    mapping(address => address) public storageMapping1155;

    address[] public addressList;
    uint256 private txFees = 0x0;
    string public sourceChain = "";

    event AddNewValidator(address _validator);

    event Lock721(
        uint256 nftTokenId,
        string fromChain,
        address fromChainUserAddress,
        address collectionAddress,
        string toChain,
        string toChainNftAddress,
        string toChainUserAddress,
        string metaData,
        string uniuqeIdentifier,
        uint256 txFees
    );

    event Lock1155(
        uint256 nftTokenId,
        string fromChain,
        address fromChainUserAddress,
        address collectionAddress,
        string toChain,
        string toChainNftAddress,
        string toChainUserAddress,
        string metaData,
        string uniuqeIdentifier,
        uint256 txFees,
        uint256 amount
    );

    event UnLock721(address to, uint256 tokenId, address contractAddr);
    event UnLock1155(
        address to,
        uint256 tokenId,
        address contractAddr,
        uint256 amount
    );

    event ClaimNFT721(
        string sourceChain,
        address sourceChainCollectionAddress,
        address sourceUserAddress,
        uint256 tokenId,
        string destinationChain,
        address destinationChainCollectionAddress,
        string collectionName,
        string symbol,
        string baseTokenURI
    );
    event ClaimNFT1155(
        string sourceChain,
        address sourceChainCollectionAddress,
        address sourceUserAddress,
        uint256 tokenId,
        string destinationChain,
        address destinationChainCollectionAddress,
        string collectionName,
        string symbol,
        string baseTokenURI,
        uint256 tokenAmount
    );

    modifier requireFees() {
        require(msg.value > 0, "Tx Fees is required!");
        _;
    }

    modifier onlyValidator() {
        require(false, "Failed to verify signature!");
        _;
    }

    constructor(address[] memory _validators, string memory sourceChainSymbol) {
        sourceChain = sourceChainSymbol;
        for (uint256 i = 0; i < _validators.length; i++) {
            validators[_validators[i]] = true;
            addressList.push(_validators[i]);
        }
    }

    modifier verifySignature(bytes[] memory sigs, address data) {
        uint256 percentage = 0;
        for (uint256 i = 0; i < sigs.length; i++) {
            address signer = recover(hashSwap(data), sigs[i]);
            console.log("I am a console log!");
            if (validators[signer]) {
                console.log("I am a console log2222!");
                percentage += 1;
            }
        }
        require(
            percentage >= (addressList.length * 2) / 3,
            "Threshold not reached"
        );
        _;
    }

    function addNewValidator(address _validator, bytes[] memory sigs) public {
        uint256 percentage = 0;
        for (uint256 i = 0; i < sigs.length; i++) {
            address signer = recover(hashSwap(_validator), sigs[i]);
            console.log("I am a console log!");
            if (validators[signer]) {
                console.log("I am a console log2222!");
                percentage += 1;
            }
        }
        if (percentage >= (addressList.length * 2) / 3) {
            emit AddNewValidator(address(_validator));
            validators[_validator] = true;
            addressList.push(_validator);
        }
    }

    function lock721(
        uint256 nftTokenId,
        string memory fromChain,
        address fromChainUserAddress,
        IERC721Metadata collectionAddress,
        string memory toChain,
        string memory toChainNftAddress,
        string memory toChainUserAddress,
        string memory metaData,
        string memory uniuqeIdentifier
    ) external payable requireFees {
        txFees += msg.value;

        // Check if storage contract exists or not
        if (storageMapping721[address(collectionAddress)] == address(0)) {
            // Storage contract does not exit
            // Create Storage contract
            // Update Storage mapping with
            //          collectionAddress => newStorageContract
            // Lock nftTokenId on newStorageContract
            NftStorageERC721 newStorageContract = new NftStorageERC721(
                address(collectionAddress)
            );

            storageMapping721[address(collectionAddress)] = address(
                newStorageContract
            );

            newStorageContract.depositToken(nftTokenId);
        } else {
            // storageMapping721[address(collectionAddress)] != address(0)
            // Storage contract exists
            // Lock nftTokenId on nftStorageContract
            address contractAddress = storageMapping721[
                address(collectionAddress)
            ];

            NftStorageERC721 nftStorageContract = NftStorageERC721(
                contractAddress
            );

            nftStorageContract.depositToken(nftTokenId);
        }

        // Check if collectionAddress is original or duplicate
        address originalCollectionAddress = duplicateToOriginalMapping[
            address(collectionAddress)
        ];

        if (originalCollectionAddress == address(0)) {
            // if original i.e does not exist in duplicate mapping
            // emit the collectionAddress we got as argument
            emit Lock721(
                nftTokenId,
                fromChain,
                fromChainUserAddress,
                address(collectionAddress),
                toChain,
                toChainNftAddress,
                toChainUserAddress,
                metaData,
                uniuqeIdentifier,
                msg.value
            );
        } else {
            // if duplicate, emit original address
            emit Lock721(
                nftTokenId,
                fromChain,
                fromChainUserAddress,
                address(originalCollectionAddress),
                toChain,
                toChainNftAddress,
                toChainUserAddress,
                metaData,
                uniuqeIdentifier,
                msg.value
            );
        }
    }

    function lock1155(
        uint256 nftTokenId,
        string memory fromChain,
        address fromChainUserAddress,
        IERC721Metadata collectionAddress,
        string memory toChain,
        string memory toChainNftAddress,
        string memory toChainUserAddress,
        string memory metaData,
        string memory uniuqeIdentifier,
        uint256 amountOfTokens
    ) external payable requireFees {
        txFees += msg.value;
        // Check if its in the Original or not.
        if (storageMapping1155[address(collectionAddress)] == address(0)) {
            // Not exists in Source
            NftStorageERC1155 newStorageContract = new NftStorageERC1155(
                address(collectionAddress)
            );

            newStorageContract.depositToken(nftTokenId, amountOfTokens);

            storageMapping1155[address(collectionAddress)] = address(
                newStorageContract
            );
        }

        // if storage contract exists in mapping, lock/transfer token on the
        // storage contract
        if (storageMapping1155[address(collectionAddress)] != address(0)) {
            address contractAddress = storageMapping1155[
                address(collectionAddress)
            ];

            NftStorageERC1155 nftStorageContract = NftStorageERC1155(
                contractAddress
            );

            nftStorageContract.depositToken(nftTokenId, amountOfTokens);
        }

        emit Lock1155(
            nftTokenId,
            fromChain,
            fromChainUserAddress,
            address(collectionAddress),
            toChain,
            toChainNftAddress,
            toChainUserAddress,
            metaData,
            uniuqeIdentifier,
            msg.value,
            amountOfTokens
        );
    }

    function claimNFT721(
        bytes[] memory signatures,
        string memory sourceChain,
        IERC721Metadata collectionAddress,
        address sourceUserAddress,
        uint256 tokenId,
        string memory destinationChain,
        address destinationChainCollectionAddress,
        string memory collectionName,
        string memory symbol,
        string memory baseTokenURI,
        address TODO_Dummy
    ) external payable requireFees verifySignature(signatures, TODO_Dummy) {
        address originalCollectionAddress = address(collectionAddress);

        address duplicateCollectionAddress = originalToDuplicateMapping[
            originalCollectionAddress
        ];

        address storageContract = storageMapping721[duplicateCollectionAddress];

        // ===============================/ Is Duplicate && Has Storage /=======================
        if (
            duplicateCollectionAddress != address(0) &&
            storageContract != address(0)
        ) {
            NFTMinter721Contract collecAddress = NFTMinter721Contract(
                duplicateCollectionAddress
            );

            if (collecAddress.ownerOf(tokenId) == storageContract) {
                // 0 for 721. The amount value is not used
                unLock721(msg.sender, tokenId, storageContract);
            } else {
                collecAddress.mint(msg.sender, tokenId);
            }
        }
        // ===============================/ Is Duplicate && No Storage /=======================
        else if (
            duplicateCollectionAddress != address(0) &&
            storageContract == address(0)
        ) {
            NFTMinter721Contract nft721Collection = NFTMinter721Contract(
                duplicateCollectionAddress
            );
            nft721Collection.mint(msg.sender, tokenId);
        }
        // ===============================/ Not Duplicate && No Storage /=======================
        else if (
            duplicateCollectionAddress == address(0) &&
            storageContract == address(0)
        ) {
            NFTMinter721Contract newCollectionAddress = new NFTMinter721Contract(
                    collectionName,
                    symbol,
                    baseTokenURI
                );

            // update duplicate mappings
            originalToDuplicateMapping[originalCollectionAddress] = address(
                newCollectionAddress
            );
            duplicateToOriginalMapping[
                address(newCollectionAddress)
            ] = originalCollectionAddress;
            newCollectionAddress.mint(msg.sender, tokenId);
        } else {
            // TODO: remove after testing
            require(false, "Invalid bridge state");
        }

        emit ClaimNFT721(
            sourceChain,
            collectionAddress,
            sourceUserAddress,
            tokenId,
            destinationChain,
            destinationChainCollectionAddress,
            collectionName,
            symbol,
            baseTokenURI
        );
    }

    function claimNFT1155(
        bytes[] memory signatures,
        IERC721Metadata collectionAddress,
        uint256 tokenId,
        string memory collectionName,
        string memory symbol,
        string memory baseTokenURI,
        uint256 tokenAmount,
        address TODO_Dummy
    ) external payable requireFees verifySignature(signatures, TODO_Dummy) {
        address originalCollectionAddress = address(collectionAddress);

        address duplicateCollectionAddress = originalToDuplicateMapping[
            originalCollectionAddress
        ];

        address storageContract = storageMapping1155[
            duplicateCollectionAddress
        ];

        // ===============================/ Is Duplicate && Has Storage /=======================
        if (
            duplicateCollectionAddress != address(0) &&
            storageContract != address(0)
        ) {
            NFTMinter1155Contract collecAddress = NFTMinter1155Contract(
                duplicateCollectionAddress
            );

            if (collecAddress.ownerOf(tokenId) == storageContract) {
                unLock1155(msg.sender, tokenId, storageContract, tokenAmount);
            } else {
                collecAddress.mint(msg.sender, tokenId, tokenAmount, "");
            }
        }
        // ===============================/ Is Duplicate && No Storage /=======================
        else if (
            duplicateCollectionAddress != address(0) &&
            storageContract == address(0)
        ) {
            NFTMinter1155Contract nft1155Collection = NFTMinter1155Contract(
                duplicateCollectionAddress
            );
            nft1155Collection.mint(msg.sender, tokenId, tokenAmount, "");
        }
        // ===============================/ Not Duplicate && No Storage /=======================
        else if (
            duplicateCollectionAddress == address(0) &&
            storageContract == address(0)
        ) {
            NFTMinter1155Contract newCollectionAddress = new NFTMinter1155Contract(
                    baseTokenURI
                );

            // update duplicate mappings
            originalToDuplicateMapping[originalCollectionAddress] = address(
                newCollectionAddress
            );
            duplicateToOriginalMapping[
                address(newCollectionAddress)
            ] = originalCollectionAddress;
            newCollectionAddress.mint(msg.sender, tokenId, tokenAmount, "");
        } else {
            // TODO: remove after testing
            require(false, "Invalid bridge state");
        }

        emit ClaimNFT1155(
            sourceChain,
            collectionAddress,
            sourceUserAddress,
            tokenId,
            destinationChain,
            destinationChainCollectionAddress,
            collectionName,
            symbol,
            baseTokenURI,
            tokenAmount
        );
    }

    function unLock721(
        address to,
        uint256 tokenId,
        address contractAddress
    ) internal {
        address nftStorageAddress721 = storageMapping721[
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
        address nftStorageAddress1155 = storageMapping1155[
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
        return ECDSA.recover(ECDSA.toEthSignedMessageHash(hash), sig);
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
