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

    mapping(address => bool) public validators;
    mapping(bytes32 => uint8) public signatureCount;

    uint256 private txFees = 0x0;
    mapping(string => bool) private tx_ids;
    uint8 public requiredConfirmations = 2;

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
        for (uint256 i = 0; i < _validators.length; i++) {
            validators[_validators[i]] = true;
        }
    }

    function addNewValidator(address _validator) public {
        emit AddNewValidator(address(_validator));
        validators[_validator] = true;
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

    function validatorSignature(bytes32 hash, bytes memory sig) external {
        require(validators[msg.sender], "Not a valid validator");
        require(!hasConfirmed(hash, msg.sender), "Signature already confirmed");

        address signer = ECDSA.recover(ECDSA.toEthSignedMessageHash(hash), sig);
        require(validators[signer], "Not a valid validator");

        bytes32 sigHash = keccak256(abi.encodePacked(hash, msg.sender));
        signatureCount[sigHash]++;

        require(signatureCount[sigHash] * 3 >= 11 * 2, "Not enough valid");
        if (signatureCount[sigHash] * 3 >= 11 * 2) {
            resetSignatureCount(hash, msg.sender);
        }
    }

    function hasConfirmed(
        bytes32 hash,
        address validator
    ) internal view returns (bool) {
        bytes32 sigHash = keccak256(abi.encodePacked(hash, validator));
        return signatureCount[sigHash] > 0;
    }

    function resetSignatureCount(bytes32 hash, address validator) internal {
        bytes32 sigHash = keccak256(
            abi.encodePacked(hash, validators[msg.sender])
        );
    }
}
