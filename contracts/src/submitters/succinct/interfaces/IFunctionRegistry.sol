// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IFunctionRegistry {
    function verifiers(bytes32 functionId) external view returns (address verifier);
    function verifierOwners(bytes32 functionId) external view returns (address owner);
    function registerFunction(address owner, address verifier, bytes32 salt)
        external
        returns (bytes32 functionId);
    function deployAndRegisterFunction(address owner, bytes memory bytecode, bytes32 salt)
        external
        returns (bytes32 functionId, address verifier);
    function updateFunction(address verifier, bytes32 salt) external returns (bytes32 functionId);
    function deployAndUpdateFunction(bytes memory bytecode, bytes32 salt)
        external
        returns (bytes32 functionId, address verifier);
    function getFunctionId(address owner, bytes32 salt)
        external
        pure
        returns (bytes32 functionId);
}