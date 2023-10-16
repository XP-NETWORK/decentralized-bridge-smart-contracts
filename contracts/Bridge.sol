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
        string to,
        uint256 tokenId,
        address contractAddr,
        string chainName,
        string tokenData,
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
        IERC721Metadata erc721Contract,
        string memory to,
        uint256 tokenId,
        string memory chainName
    ) external payable {
        txFees += msg.value;
        emit Lock(
            to,
            tokenId,
            address(erc721Contract),
            chainName,
            erc721Contract.tokenURI(tokenId),
            msg.value
        );
        erc721Contract.safeTransferFrom(
            address(msg.sender),
            address(this),
            tokenId
        );
    }

    function unLock(
        address to,
        uint256 tokenId,
        IERC721 contractAddr
    ) external payable {
        emit UnLock(to, tokenId, address(contractAddr));
        contractAddr.safeTransferFrom(address(this), to, tokenId);
    }

    modifier requireFees() {
        require(msg.value > 0, "Tx Fees is required!");
        _;
    }
}
