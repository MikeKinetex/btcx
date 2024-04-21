// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library HeadersInput {
    function get(bytes calldata headers, uint256 index) internal pure returns (bytes calldata header) {
        return headers[80 * index:80 * (index + 1)];
    }

    function slice(
        bytes calldata headers,
        uint256 startIndex,
        uint256 endIndex
    ) internal pure returns (bytes calldata header) {
        return headers[80 * startIndex:80 * endIndex];
    }

    function size(bytes calldata headers) internal pure returns (uint256) {
        return headers.length / 80;
    }
}
