// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ILightClient} from "../../interfaces/ILightClient.sol";
import {ISuccinctGateway} from "./interfaces/ISuccinctGateway.sol";
import {OutputReader} from "./lib/OutputReader.sol";

contract SuccinctSubmitter {
    error InvalidGatewayCall(address caller);
    error NoFunctionId(uint256 nHeaders, bool needRetarget);

    struct VerifyFunction {
        bytes32 functionId;
        uint64 nHeaders;
    }

    ILightClient private immutable LIGHT_CLIENT;
    ISuccinctGateway private immutable SUCCINCT_GATEWAY;

    mapping(uint64 => bytes32) private verifyFunctionIds;
    mapping(uint64 => bytes32) private retargetFunctionIds;

    constructor(
        address lightClient,
        address gateway,
        VerifyFunction[] memory _verifyFunctions,
        VerifyFunction[] memory _retargetFunctions
    ) {
        LIGHT_CLIENT = ILightClient(lightClient);
        SUCCINCT_GATEWAY = ISuccinctGateway(gateway);

        for (uint256 i = 0; i < _verifyFunctions.length; i++) {
            verifyFunctionIds[_verifyFunctions[i].nHeaders] = _verifyFunctions[i].functionId;
        }
        for (uint256 i = 0; i < _retargetFunctions.length; i++) {
            retargetFunctionIds[_retargetFunctions[i].nHeaders] = _retargetFunctions[i].functionId;
        }
    }

    function request(bytes32 parentBlockHash, uint64 nHeaders) external payable {
        uint64 parentBlockHeight = LIGHT_CLIENT.blockByHash(parentBlockHash);
        uint256 currentTarget = LIGHT_CLIENT.targetByHeight(parentBlockHeight);

        uint64 startPeriodBlockHeight = parentBlockHeight - (parentBlockHeight % 2016);
        uint64 endPeriodBlockHeight = startPeriodBlockHeight + 2015;

        bytes32 startPeriodBlockHash = LIGHT_CLIENT.blockByHeight(startPeriodBlockHeight);

        bool needRetarget = parentBlockHeight + nHeaders > endPeriodBlockHeight;

        bytes memory input = needRetarget
            ? abi.encodePacked(parentBlockHeight, parentBlockHash, startPeriodBlockHash, currentTarget)
            : abi.encodePacked(parentBlockHash, currentTarget);

        bytes32 functionId = needRetarget ? retargetFunctionIds[nHeaders] : verifyFunctionIds[nHeaders];
        if (functionId == 0) revert NoFunctionId(nHeaders, needRetarget);

        SUCCINCT_GATEWAY.requestCallback{value: msg.value}(
            functionId,
            input,
            abi.encode(parentBlockHash, nHeaders, needRetarget),
            this.submit.selector,
            300000
        );
    }

    function submit(bytes memory output, bytes memory context) external {
        if (msg.sender != address(SUCCINCT_GATEWAY)) revert InvalidGatewayCall(msg.sender);
        if (!SUCCINCT_GATEWAY.isCallback()) revert InvalidGatewayCall(msg.sender);

        (bytes32 parentBlockHash, uint64 nHeaders, bool needRetarget) = abi.decode(context, (bytes32, uint64, bool));

        bytes32[] memory blockHashes = new bytes32[](nHeaders);
        for (uint256 i = 0; i < nHeaders; i++) {
            blockHashes[i] = bytes32(OutputReader.readUint256(output, i * 32));
        }

        if (needRetarget) {
            uint256 nextTarget = OutputReader.readUint256(output, nHeaders * 32);
            LIGHT_CLIENT.submit(parentBlockHash, blockHashes, nextTarget);
        } else {
            LIGHT_CLIENT.submit(parentBlockHash, blockHashes);
        }
    }
}
