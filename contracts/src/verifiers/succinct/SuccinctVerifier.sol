// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IVerifier} from "../../interfaces/IVerifier.sol";

contract SuccinctVerifier is IVerifier {
    function verify(
        uint64 /* ancestorBlockHeight */,
        bytes32 /* ancestorBlockHash */,
        uint256 /* currentTarget */,
        bytes calldata /* proof */
    ) external pure returns (bytes32) {
        revert BlockVerificationNotSupported();
    }

    function verifyWithRetargeting(
        uint64 /* ancestorBlockHeight */,
        bytes32 /* ancestorBlockHash */,
        bytes32 /* startPeriodHash */,
        uint256 /* target */,
        bytes calldata /* proof */
    ) external pure returns (uint256[] memory, bytes32) {
        revert RetargetingNotSupported();
    }

    function verifyUtreexo(
        bytes32 /* blockHash */,
        bytes32 /* parentUtreexo */,
        bytes calldata /* proof */
    ) external pure returns (bytes32[] memory) {
        revert UtreexoNotSupported();
    }
}
