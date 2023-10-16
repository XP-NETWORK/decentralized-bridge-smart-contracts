// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.19 <0.9.0;

import "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "@openzeppelin/contracts/token/ERC721/utils/ERC721Holder.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/IERC721Metadata.sol";

contract Bridge is ERC721Holder {
    address[] public validators;

    event AddNewValidator(address _validator);
    event Lock(
        string to,
        uint256 tokenId,
        address contractAddr,
        uint256 chainNonce
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
        string memory to,
        uint256 tokenId,
        IERC721Metadata erc721Contract,
        uint256 chainNonce
    ) external payable {
        emit Lock(to, tokenId, address(erc721Contract), chainNonce);
        erc721Contract.safeTransferFrom(msg.sender, address(this), tokenId);
    }

    function unLock(
        address to,
        uint256 tokenId,
        IERC721 contractAddr
    ) external {
        emit UnLock(to, tokenId, address(contractAddr));
        contractAddr.safeTransferFrom(address(this), to, tokenId);
    }
}
