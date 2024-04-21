// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library BitcoinHeader {
    function hash(bytes calldata header) internal pure returns (bytes32) {
        return sha256(abi.encode(sha256(header)));
    }

    function version(bytes calldata header) internal pure returns (uint32) {
        return reverse32(uint32(bytes4(header[:4])));
    }

    function parentHash(bytes calldata header) internal pure returns (bytes32) {
        return bytes32(header[4:36]);
    }

    function merkleRoot(bytes calldata header) internal pure returns (bytes32) {
        return bytes32(header[36:68]);
    }

    function timestamp(bytes calldata header) internal pure returns (uint32) {
        return reverse32(uint32(bytes4(header[68:72])));
    }

    function nBits(bytes calldata header) internal pure returns (uint32) {
        return reverse32(uint32(bytes4(header[72:76])));
    }

    function nonce(bytes calldata header) internal pure returns (uint32) {
        return reverse32(uint32(bytes4(header[76:80])));
    }

    function reverse32(uint32 input) internal pure returns (uint32 v) {
        v = input;
        v = ((v & 0xFF00FF00) >> 8) | ((v & 0x00FF00FF) << 8);
        v = (v >> 16) | (v << 16);
    }
}
