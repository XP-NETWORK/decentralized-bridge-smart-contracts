// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "./interfaces/INFTClaim.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract HederaNFTClaim is INFTClaim, Ownable {
    using EnumerableSet for EnumerableSet.UintSet;

    event ClaimCreated(
        address indexed to,
        address indexed token,
        int64 indexed serial
    );
    event ClaimRemoved(
        address indexed to,
        address indexed token,
        int64 indexed serial
    );

    constructor() Ownable(msg.sender) {}

    mapping(address => mapping(address => EnumerableSet.UintSet))
        private nftClaims;

    function getClaimableNfts(
        address claimer,
        address token
    ) public view returns (uint256[] memory) {
        return nftClaims[claimer][token].values();
    }

    function decodeHts(uint256 data) public pure returns (address, int64) {
        bytes32 d2 = bytes32(data);
        address token = address(uint160(bytes20(d2)));
        int96 serialNum = int96(uint96(uint256(d2)));

        return (token, int64(serialNum));
    }

    function getClaimRecord(
        address sender,
        address token,
        int64 serialNum
    ) external onlyOwner {
        EnumerableSet.UintSet storage serialNums = nftClaims[sender][token];

        require(
            serialNums.remove(uint256(uint64(serialNum))),
            "Cannot claim this nft"
        );
        emit ClaimRemoved(sender, token, serialNum);
    }

    function addClaimRecord(
        address to,
        address token,
        int64 serial
    ) external onlyOwner {
        nftClaims[to][token].add(uint256(int256(serial)));
        emit ClaimCreated(to, token, serial);
    }
}
