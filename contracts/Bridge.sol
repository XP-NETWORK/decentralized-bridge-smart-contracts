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
    mapping(string => address) public signerAddress;
    uint256 private txFees = 0x0;
    uint256 private validatorsConfirmCount = 0;

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
        string memory uniuqeIdentifier,
        bytes32 hash,
        bytes memory sig
    ) external payable requireFees {
        require(
            confirmSignature(hash, sig),
            "validator signature is not valid!"
        );
        if (validatorsConfirmCount >= 7) {
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
            validatorsConfirmCount = 0;
        }
    }

    function unLock(
        address to,
        uint256 tokenId,
        IERC721 contractAddr,
        bytes32 hash,
        bytes memory sig
    ) external payable requireFees {
        require(
            confirmSignature(hash, sig),
            "validator signature is not valid!"
        );
        if (validatorsConfirmCount >= 7) {
            emit UnLock(to, tokenId, address(contractAddr));
            contractAddr.safeTransferFrom(address(this), to, tokenId);
            validatorsConfirmCount = 0;
        }
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

    function confirmSignature(
        bytes32 hash,
        bytes memory sig
    ) internal returns (bool) {
        validatorsConfirmCount += 1;
        return
            validators[ECDSA.recover(ECDSA.toEthSignedMessageHash(hash), sig)];
    }
}
