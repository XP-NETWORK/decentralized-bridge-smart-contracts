// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface INFTClaim {
    function getClaimableNfts(
        address claimer,
        address token
    ) external view returns (uint256[] memory);

    function decodeHts(uint256 data) external pure returns (address, int64);

    function getClaimRecord(
        address sender,
        address token,
        int64 serialNum
    ) external;

    function addClaimRecord(address to, address token, int64 serial) external;
}
