// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface ISuccinctGateway {
    function requestCallback(
        bytes32 functionId,
        bytes memory input,
        bytes memory context,
        bytes4 callbackSelector,
        uint32 callbackGasLimit
    ) external payable returns (bytes32);

    function requestCall(
        bytes32 functionId,
        bytes memory input,
        address entryAddress,
        bytes memory entryData,
        uint32 entryGasLimit
    ) external payable;

    function verifiedCall(bytes32 functionId, bytes memory input) external view returns (bytes memory);

    function isCallback() external view returns (bool);
}
