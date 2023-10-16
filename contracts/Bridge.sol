// // SPDX-License-Identifier: UNLICENSED
// pragma solidity ^0.8.19;

// import "@openzeppelin/contracts/token/ERC721/IERC721.sol";

// // Uncomment this line to use console.log
// // import "hardhat/console.sol";

// struct lockNft {
//     uint256 id;
//     uint256 contractAddress;
//     uint256 chainId;
//     address payable owner;
// }

// contract Bridge {
//     address[] public validators;
//     lockNft[] public lockNfts;

//     constructor(address[] memory _validators) {
//         validators = _validators;
//     }

//     function returnValidators() public view returns (address[] memory) {
//         return validators;
//     }

//     function addNewValidator(address _validator) public {
//         validators.push(_validator);
//     }

//     function lockNft(
//         address to,
//         uint256 tokenId,
//         IERC721 contractAddr
//     ) public {
//         require(msg.sender == owner, "Only the owner can lock the NFT");
//         contractAddr.safeTransferFrom(address(this),to, tokenId);
//     }
// }
