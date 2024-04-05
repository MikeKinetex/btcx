// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library ProofDecoder {
    function getHeader(bytes calldata proof, uint256 index) internal pure returns (bytes calldata header) {
        return proof[80 * index:80 * (index + 1)];
    }

    function size(bytes calldata proof) internal pure returns (uint256) {
        return proof.length / 80;
    }
}
