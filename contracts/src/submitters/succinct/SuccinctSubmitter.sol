// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ILightClient} from "../../interfaces/ILightClient.sol";
import {ISuccinctGateway} from "./interfaces/ISuccinctGateway.sol";
import {OutputReader} from "./lib/OutputReader.sol";

contract SuccinctSubmitter {
    error NoFunctionId(uint256 nHeaders, bool needRetarget);

    struct ZKFunction {
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
        ZKFunction[] memory _verifyFunctions,
        ZKFunction[] memory _retargetFunctions
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
        (bytes32 functionId, bytes memory input) = _constructZKCall(parentBlockHash, nHeaders);
        SUCCINCT_GATEWAY.requestCall{value: msg.value}(
            functionId,
            input,
            address(this),
            abi.encodeWithSelector(this.submit.selector, parentBlockHash, nHeaders),
            10000000
        );
    }

    function submit(bytes32 parentBlockHash, uint64 nHeaders) external {
        (bytes32 functionId, bytes memory input) = _constructZKCall(parentBlockHash, nHeaders);

        bytes memory output = SUCCINCT_GATEWAY.verifiedCall(functionId, input);

        bytes32[] memory blockHashes = new bytes32[](nHeaders);
        for (uint256 i = 0; i < nHeaders; i++) {
            blockHashes[i] = bytes32(OutputReader.readUint256(output, i * 32));
        }

        if (functionId == retargetFunctionIds[nHeaders]) {
            uint256 nextTarget = OutputReader.readUint256(output, nHeaders * 32);
            LIGHT_CLIENT.submit(parentBlockHash, blockHashes, nextTarget);
        } else {
            LIGHT_CLIENT.submit(parentBlockHash, blockHashes);
        }
    }

    function _constructZKCall(bytes32 parentBlockHash, uint64 nHeaders) internal view returns (bytes32, bytes memory) {
        uint64 parentBlockHeight = LIGHT_CLIENT.blockByHash(parentBlockHash);
        uint256 currentTarget = LIGHT_CLIENT.targetByHeight(parentBlockHeight);

        uint64 startPeriodBlockHeight = parentBlockHeight - (parentBlockHeight % 2016);
        uint64 endPeriodBlockHeight = startPeriodBlockHeight + 2015;

        bytes32 startPeriodBlockHash = LIGHT_CLIENT.blockByHeight(startPeriodBlockHeight);

        bool needRetarget = parentBlockHeight + nHeaders > endPeriodBlockHeight;

        bytes32 functionId = needRetarget ? retargetFunctionIds[nHeaders] : verifyFunctionIds[nHeaders];
        if (functionId == 0) revert NoFunctionId(nHeaders, needRetarget);

        bytes memory input = needRetarget
            ? abi.encodePacked(parentBlockHeight, parentBlockHash, startPeriodBlockHash, currentTarget)
            : abi.encodePacked(parentBlockHash, currentTarget);

        return (functionId, input);
    }
}
