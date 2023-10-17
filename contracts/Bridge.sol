// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.19 <0.9.0;

import "hardhat/console.sol";

import "@openzeppelin/contracts/token/ERC721/extensions/IERC721Metadata.sol";
import "@openzeppelin/contracts/token/ERC721/utils/ERC721Holder.sol";
import "@openzeppelin/contracts/token/ERC1155/extensions/IERC1155MetadataURI.sol";
import "@openzeppelin/contracts/token/ERC1155/utils/ERC1155Holder.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract Bridge is ERC721Holder, ERC1155Holder {
    using ECDSA for bytes32;

    struct Signature {
        bytes32 messageHash;
        bytes signature;
    }

    address[] public validator;
    uint256 private txFees = 0x0;
    mapping(string => bool) private tx_ids;
    mapping(address => Signature) public userSignatures;

    event AddNewValidator(address _validator);
    event Lock(
        uint256 nft_token_id,
        string from_chain,
        address from_chain_user_address,
        address from_chain_nft_address,
        string to_chain,
        string to_chain_nft_address,
        string to_chain_user_address,
        string meta_data,
        string uniuqeIdentifier,
        uint256 txFees
    );

    event UnLock(address to, uint256 tokenId, address contractAddr);

    constructor(address[] memory _validators) {
        validator = _validators;
    }

    function returnValidators() public view returns (address[] memory) {
        return validator;
    }

    function addNewValidator(address _validator) public {
        emit AddNewValidator(address(_validator));
        return validator.push(_validator);
    }

    function lock(
        uint256 nft_token_id,
        string memory from_chain,
        address from_chain_user_address,
        IERC721Metadata from_chain_nft_address,
        string memory to_chain,
        string memory to_chain_nft_address,
        string memory to_chain_user_address,
        string memory meta_data,
        string memory uniuqeIdentifier
    ) external payable requireFees {
        txFees += msg.value;
        emit Lock(
            nft_token_id,
            from_chain,
            from_chain_user_address,
            address(from_chain_nft_address),
            to_chain,
            to_chain_nft_address,
            to_chain_user_address,
            meta_data,
            uniuqeIdentifier,
            msg.value
        );
        from_chain_nft_address.safeTransferFrom(
            address(msg.sender),
            address(this),
            nft_token_id
        );
    }

    function unLock(
        address to,
        uint256 tokenId,
        IERC721 contractAddr
    ) external payable requireFees {
        emit UnLock(to, tokenId, address(contractAddr));
        contractAddr.safeTransferFrom(address(this), to, tokenId);
    }

    modifier requireFees() {
        require(msg.value > 0, "Tx Fees is required!");
        _;
    }

    function claimFee(address payable receiver) external {
        uint256 sendAmt = txFees;
        txFees = 0;
        receiver.transfer(sendAmt);
    }

    function storeSignature(
        bytes32 _messageHash,
        bytes memory _signature
    ) public {
        userSignatures[msg.sender] = Signature(_messageHash, _signature);
    }

    function verifySignature(
        bytes32 _messageHash,
        bytes memory _signature,
        address _signer
    ) public pure returns (bool) {
        address signer = _messageHash.recover(_signature);
        return signer == _signer;
    }

    function executeAction(
        bytes32 _messageHash,
        bytes memory _signature
    ) public {
        require(validator.length > 0, "No validators registered.");
        uint256 validSignatureCount = 0;

        for (uint256 i = 0; i < validator.length; i++) {
            if (isValidatorSignature(validator[i], _signature, _messageHash)) {
                validSignatureCount++;
            }
        }

        require(
            validSignatureCount * 3 >= validator.length * 2,
            "Not enough valid signatures."
        );
    }

    function isValidatorSignature(
        address _validator,
        bytes memory _signature,
        bytes32 _messageHash
    ) internal view returns (bool) {
        require(checkIfValidatorExists(_validator), "Not a validator.");

        // Verify the signature using ECDSA
        bytes32 ethSignedMessageHash = ECDSA.toEthSignedMessageHash(
            _messageHash
        );
        address recoveredAddress = ECDSA.recover(
            ethSignedMessageHash,
            _signature
        );

        return recoveredAddress == _validator;
    }

    function checkIfValidatorExists(
        address _validator
    ) internal view returns (bool) {
        for (uint256 i = 0; i < validator.length; i++) {
            if (validator[i] == _validator) {
                return true;
            }
        }
        return false;
    }
}
