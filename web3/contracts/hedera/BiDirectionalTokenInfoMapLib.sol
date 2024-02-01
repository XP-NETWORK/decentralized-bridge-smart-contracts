// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

struct TokenInfo {
    uint256 tokenId;
    string chain;
    string contract_;
    bool exists;
}

library BiDirectionalTokenInfoMapLib {
    function insert(
        TokenInfo memory id,
        TokenInfo memory value,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage idToValue,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage valueToId
    ) internal {
        idToValue[id.chain][id.contract_][id.tokenId] = value;
        valueToId[value.chain][value.contract_][value.tokenId] = id;
    }

    function containsValue(
        TokenInfo memory value,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage valueToId
    ) internal view returns (bool) {
        return valueToId[value.chain][value.contract_][value.tokenId].exists;
    }

    function getValue(
        TokenInfo memory id,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage idToValue,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage
    ) internal view returns (TokenInfo memory) {
        return idToValue[id.chain][id.contract_][id.tokenId];
    }

    function containsId(TokenInfo memory id,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage idToValue,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage) internal view returns (bool) {
        return idToValue[id.chain][id.contract_][id.tokenId].exists;
    }

    function getId(
        TokenInfo memory value,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage,
        mapping(string => mapping(string => mapping(uint256 => TokenInfo)))
            storage valueToId
    ) internal view returns (TokenInfo memory) {
        return valueToId[value.chain][value.contract_][value.tokenId];
    }
}
