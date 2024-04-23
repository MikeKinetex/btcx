// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface ISuccinctSubmitter {
    struct ZKFunction {
        bytes32 functionId;
        uint64 nHeaders;
    }

    error NoFunctionId(uint256 nHeaders, bool needRetarget);
    error InvalidProof(address verifier, bytes32 inputHash, bytes32 outputHash, bytes proof);

    function request(bytes32 parentBlockHash, uint64 nHeaders) external payable;

    function submit(bytes32 parentBlockHash, uint64 nHeaders) external;

    function submit(bytes32 parentBlockHash, uint64 nHeaders, bytes memory output, bytes memory proof) external;
}
