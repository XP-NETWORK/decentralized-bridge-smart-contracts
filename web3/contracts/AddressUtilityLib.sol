// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title AddressUtilityLib
 * @dev A library for converting string representations of Ethereum addresses.
 */
library AddressUtilityLib {
    /**
     * @dev Converts a string to an Ethereum address.
     * @param _address The string representation of an Ethereum address.
     * @return addr The converted Ethereum address.
     */
    function stringToAddress(
        string memory _address
    ) internal pure returns (address addr) {
        bytes memory strBytes = bytes(_address);
        require(
            strBytes.length == 42 || strBytes.length == 40,
            "Invalid address length"
        );

        // If the address starts with '0x', remove it
        if (strBytes.length == 42) {
            require(
                strBytes[0] == bytes1("0") && strBytes[1] == bytes1("x"),
                "Missing '0x' prefix"
            );
            strBytes = bytes(substring(_address, 2, 42));
        }

        uint160 y = 0;
        for (uint i = 0; i < strBytes.length; i++) {
            uint8 b = uint8(strBytes[i]);
            if (b >= 48 && b <= 57) {
                b -= 48; // '0' to '9'
            } else if (b >= 97 && b <= 102) {
                b -= 87; // 'a' to 'f'
            } else if (b >= 65 && b <= 70) {
                b -= 55; // 'A' to 'F'
            } else {
                revert("Invalid character in address string");
            }
            y = y * 16 + uint160(b);
        }
        return address(y);
    }

    /**
     * @dev Extracts a substring from a string.
     * @param str The original string.
     * @param startIndex The start index for the substring.
     * @param endIndex The end index for the substring.
     * @return The extracted substring.
     */
    function substring(
        string memory str,
        uint startIndex,
        uint endIndex
    ) internal pure returns (string memory) {
        bytes memory strBytes = bytes(str);
        bytes memory result = new bytes(endIndex - startIndex);
        for (uint i = startIndex; i < endIndex; i++) {
            result[i - startIndex] = strBytes[i];
        }
        return string(result);
    }

    /**
     * @dev Compares two strings for equality.
     * @param a First string.
     * @param b Second string.
     * @return True if strings are equal, false otherwise.
     */
    function compareStrings(
        string memory a,
        string memory b
    ) internal pure returns (bool) {
        return keccak256(abi.encodePacked(a)) == keccak256(abi.encodePacked(b));
    }
}
