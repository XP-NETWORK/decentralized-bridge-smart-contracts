// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.19 <0.9.0;

import "hardhat/console.sol";

import "@openzeppelin/contracts/token/ERC721/extensions/IERC721Metadata.sol";
import "@openzeppelin/contracts/token/ERC721/utils/ERC721Holder.sol";
import "@openzeppelin/contracts/token/ERC1155/extensions/IERC1155MetadataURI.sol";
import "@openzeppelin/contracts/token/ERC1155/utils/ERC1155Holder.sol";

contract Bridge is ERC721Holder, ERC1155Holder {
    address[] public validators;
    uint256 private txFees = 0x0;

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
        string unique_action_id_from_contract,
        uint256 txFees
    );
    event UnLock(address to, uint256 tokenId, address contractAddr);

    constructor(address[] memory _validators) {
        validators = _validators;
    }

    function returnValidators() public view returns (address[] memory) {
        return validators;
    }

    function addNewValidator(address _validator) public {
        emit AddNewValidator(address(_validator));
        validators.push(_validator);
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
        string memory unique_action_id_from_contract
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
            unique_action_id_from_contract,
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
}
