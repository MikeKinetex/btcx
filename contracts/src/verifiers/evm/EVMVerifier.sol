// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IVerifier} from "../../interfaces/IVerifier.sol";
import {HeadersInput} from "./lib/HeadersInput.sol";
import {BitcoinHeader} from "./lib/BitcoinHeader.sol";

contract EVMVerifier is IVerifier {
    using BitcoinHeader for bytes;
    using HeadersInput for bytes;

    function verify(
        bytes32 ancestorBlockHash,
        uint256 currentTarget,
        bytes calldata headers
    ) external pure returns (bytes32[] memory) {
        uint256 nHeaders = headers.size();
        if (nHeaders < 1 || nHeaders > 2016) revert InvalidProofLength();

        bytes32[] memory blockHashes = new bytes32[](nHeaders);

        for (uint256 i = 0; i < nHeaders; i++) {
            blockHashes[i] = _verifyHeader(
                headers.get(i),
                i == 0 ? ancestorBlockHash : blockHashes[i - 1],
                currentTarget
            );
        }

        return blockHashes;
    }

    function verifyWithRetargeting(
        uint64 ancestorBlockHeight,
        bytes32 ancestorBlockHash,
        bytes32 startPeriodBlockHash,
        uint256 currentTarget,
        bytes calldata headers
    ) external pure returns (bytes32[] memory, uint256) {
        uint256 hsz = headers.size();
        if (hsz <= 2) revert InvalidProofLength();

        uint256 nHeaders = hsz - 2;
        uint256 shift = (2016 - ((ancestorBlockHeight + 1) % 2016)) % 2016;
        if (shift < nHeaders && nHeaders - shift > 2016) revert InvalidProofLength();

        bytes32[] memory blockHashes = new bytes32[](nHeaders);

        if (shift > 0) {
            uint256 n = shift < nHeaders ? shift : nHeaders;

            for (uint256 i = 0; i < n; i++) {
                blockHashes[i] = _verifyHeader(
                    headers.get(i + 2),
                    i == 0 ? ancestorBlockHash : blockHashes[i - 1],
                    currentTarget
                );
            }

            if (shift >= nHeaders) return (blockHashes, 0);
        }

        bytes32 endPeriodBlockHash = shift > 0 ? blockHashes[shift - 1] : ancestorBlockHash;

        if (headers.get(0).hash() != startPeriodBlockHash) revert InvalidStartPeriodBlockHash();
        if (headers.get(1).hash() != endPeriodBlockHash) revert InvalidEndPeriodBlockHash();

        uint256 nextTarget = _adjustTarget(currentTarget, headers.get(0).timestamp(), headers.get(1).timestamp());

        for (uint256 i = shift; i < nHeaders; i++) {
            blockHashes[i] = _verifyHeader(
                headers.get(i + 2),
                i == shift ? endPeriodBlockHash : blockHashes[i - 1],
                nextTarget
            );
        }

        return (blockHashes, _nBitsToTarget(headers.get(shift + 2).nBits()) & nextTarget);
    }

    function verifyUtreexo(
        bytes32 /* blockHash */,
        bytes32 /* parentUtreexo */,
        bytes calldata /* proof */
    ) external pure returns (bytes32[] memory) {
        revert UtreexoNotSupported();
    }

    function _verifyHeader(
        bytes calldata blockHeader,
        bytes32 parentHash,
        uint256 currentTarget
    ) internal pure returns (bytes32) {
        // get block hash
        bytes32 blockHash = blockHeader.hash();

        // check parent hash
        if (blockHeader.parentHash() != parentHash) revert InvalidParentHash();

        // check target
        uint256 target = _nBitsToTarget(blockHeader.nBits());
        if ((target & currentTarget) != target) revert NotSamePeriod();
        if (uint256(blockHash) <= target) revert WorkBelowTarget();

        return blockHash;
    }

    function _nBitsToTarget(uint32 nBits) internal pure returns (uint256 target) {
        uint256 nSize = nBits >> 24;
        uint256 nWord = nBits & 0x007fffff;

        if (nSize <= 3) {
            nWord >>= 8 * (3 - nSize);
            target = nWord;
        } else {
            target = nWord << (8 * (nSize - 3));
        }

        bool fNegative = nWord != 0 && (nBits & 0x00800000) != 0;
        bool fOverflow = nWord != 0 && ((nSize > 34) || (nWord > 0xff && nSize > 33) || (nWord > 0xffff && nSize > 32));
        if (fNegative || fOverflow || target == 0) {
            revert InvalidTarget();
        }
    }

    function _adjustTarget(
        uint256 target,
        uint32 periodStartTime,
        uint32 periodEndTime
    ) internal pure returns (uint256 newTarget) {
        uint32 powTargetTimespan = 14 * 24 * 60 * 60;
        uint256 powLimit = 0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

        uint32 timespan = periodEndTime - periodStartTime;
        if (timespan < powTargetTimespan / 4) {
            timespan = powTargetTimespan / 4;
        }
        if (timespan > powTargetTimespan * 4) {
            timespan = powTargetTimespan * 4;
        }

        newTarget = (target * timespan) / powTargetTimespan;
        if (newTarget > powLimit) {
            newTarget = powLimit;
        }
    }
}
