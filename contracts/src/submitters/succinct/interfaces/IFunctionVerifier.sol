// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IFunctionVerifier {
    function verify(bytes32 inputHash, bytes32 outputHash, bytes memory proof)
        external
        returns (bool);

    function verificationKeyHash() external view returns (bytes32);
}