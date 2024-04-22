// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library OutputReader {
    function readUint256(bytes memory _output, uint256 _offset) internal pure returns (uint256) {
        uint256 value;
        // solhint-disable-next-line no-inline-assembly
        assembly {
            value := mload(add(add(_output, 0x20), _offset))
        }
        return value;
    }
}
