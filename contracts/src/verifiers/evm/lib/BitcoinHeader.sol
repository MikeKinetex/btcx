// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Endian} from "./Endian.sol";

library BitcoinHeader {
    using Endian for uint32;

    function hash(bytes calldata header) internal pure returns (bytes32) {
        return sha256(abi.encode(sha256(header)));
    }

    function version(bytes calldata header) internal pure returns (uint32) {
        return uint32(bytes4(header[:4])).reverse32();
    }

    function parentHash(bytes calldata header) internal pure returns (bytes32) {
        return bytes32(header[4:36]);
    }

    function merkleRoot(bytes calldata header) internal pure returns (bytes32) {
        return bytes32(header[36:68]);
    }

    function timestamp(bytes calldata header) internal pure returns (uint32) {
        return uint32(bytes4(header[68:72])).reverse32();
    }

    function nBits(bytes calldata header) internal pure returns (uint32) {
        return uint32(bytes4(header[72:76])).reverse32();
    }

    function nonce(bytes calldata header) internal pure returns (uint32) {
        return uint32(bytes4(header[76:80])).reverse32();
    }
}
