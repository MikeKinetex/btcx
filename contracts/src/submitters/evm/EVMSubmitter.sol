// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IEVMSubmitter} from "./interfaces/IEVMSubmitter.sol";
import {ILightClient} from "../../interfaces/ILightClient.sol";
import {IBlockVerifier} from "./interfaces/IBlockVerifier.sol";

contract EVMSubmitter is IEVMSubmitter {
    ILightClient private immutable LIGHT_CLIENT;
    IBlockVerifier private immutable BLOCK_VERIFIER;

    constructor(address lightClient, address blockVerifier) {
        LIGHT_CLIENT = ILightClient(lightClient);
        BLOCK_VERIFIER = IBlockVerifier(blockVerifier);
    }

    function submit(bytes32 parentBlockHash, bytes calldata headers) external {
        uint256 nHeaders = headers.length / 80;

        uint64 parentBlockHeight = LIGHT_CLIENT.blockByHash(parentBlockHash);
        uint256 currentTarget = LIGHT_CLIENT.targetByHeight(parentBlockHeight);

        uint64 startPeriodBlockHeight = parentBlockHeight - (parentBlockHeight % 2016);
        uint64 endPeriodBlockHeight = startPeriodBlockHeight + 2015;

        bytes32 startPeriodBlockHash = LIGHT_CLIENT.blockByHeight(startPeriodBlockHeight);

        if (parentBlockHeight + nHeaders > endPeriodBlockHeight) {
            // verify blocks with retargeting
            (bytes32[] memory blockHashes, uint256 nextTarget) = BLOCK_VERIFIER.verifyWithRetargeting(
                parentBlockHeight,
                parentBlockHash,
                startPeriodBlockHash,
                currentTarget,
                headers
            );
            LIGHT_CLIENT.submit(parentBlockHash, blockHashes, nextTarget);
        } else {
            // verify blocks without retargeting
            bytes32[] memory blockHashes = BLOCK_VERIFIER.verify(parentBlockHash, currentTarget, headers);
            LIGHT_CLIENT.submit(parentBlockHash, blockHashes);
        }
    }
}
