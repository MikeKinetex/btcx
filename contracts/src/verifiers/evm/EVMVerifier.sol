// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IVerifier} from "../../interfaces/IVerifier.sol";
import {ProofDecoder} from "./lib/ProofDecoder.sol";
import {BitcoinHeader} from "./lib/BitcoinHeader.sol";

contract EVMVerifier is IVerifier {
    using BitcoinHeader for bytes;

    function verify(
        uint64 /* ancestorBlockHeight */,
        bytes32 ancestorBlockHash,
        uint256 currentTarget,
        bytes calldata proof
    ) external pure returns (bytes32 endBlockHash) {
        endBlockHash = _verify(ancestorBlockHash, currentTarget, proof);
    }

    function verifyWithRetargeting(
        uint64 ancestorBlockHeight,
        bytes32 ancestorBlockHash,
        bytes32 startPeriodBlockHash,
        uint256 currentTarget,
        bytes calldata proof
    ) external pure returns (uint256[] memory retargets, bytes32 endBlockHash) {
        uint256 psz = ProofDecoder.size(proof);
        if (psz <= 2) revert InvalidProofLength();

        bytes32 parentBlockHash = ancestorBlockHash;

        uint256 shift = (2016 - ((ancestorBlockHeight + 1) % 2016)) % 2016;

        if (shift > 0) {
            bool neetRetarget = shift < psz - 2;

            parentBlockHash = _verify(parentBlockHash, currentTarget, ProofDecoder.slice(proof, 2, neetRetarget ? 2 + shift : psz));

            if (!neetRetarget) {
                return (new uint256[](0), parentBlockHash);
            }
        }

        bytes calldata startPeriodBlockHeader = ProofDecoder.get(proof, 0);
        bytes calldata endPeriodBlockHeader = ProofDecoder.get(proof, 1);
        bytes calldata headers = ProofDecoder.slice(proof, 2 + shift, psz);

        if (startPeriodBlockHeader.hash() != startPeriodBlockHash) {
            revert InvalidStartPeriodBlockHash();
        }

        if (endPeriodBlockHeader.hash() != parentBlockHash) {
            revert InvalidEndPeriodBlockHash();
        }

        (retargets, endBlockHash) = _verifyWithRetargeting(
            parentBlockHash,
            currentTarget,
            startPeriodBlockHeader.timestamp(),
            endPeriodBlockHeader.timestamp(),
            headers
        );
    }

    function verifyUtreexo(bytes32 /* blockHash */, bytes32 /* parentUtreexo */, bytes calldata /* proof */) external pure returns (bytes32[] memory) {
        revert UtreexoNotSupported();
    }

    function _verify(bytes32 ancestorBlockHash, uint256 currentTarget, bytes calldata proof) internal pure returns (bytes32) {
        uint256 nHeaders = ProofDecoder.size(proof);
        if (nHeaders < 1 || nHeaders > 2016) revert InvalidProofLength();

        bytes32 parentBlockHash = ancestorBlockHash;

        for (uint256 i = 0; i < nHeaders; i++) {
            bytes calldata blockHeader = ProofDecoder.get(proof, i);
            bytes32 blockHash = blockHeader.hash();

            // check parent hash
            if (blockHeader.parentHash() != parentBlockHash) revert InvalidParentHash();

            // check target
            uint256 target = _nBitsToTarget(blockHeader.nBits());
            if ((target & currentTarget) != target) revert NotSamePeriod();
            if (uint256(blockHash) <= target) revert WorkBelowTarget();

            parentBlockHash = blockHash;
        }

        return parentBlockHash;
    }

    function _verifyWithRetargeting(
        bytes32 ancestorBlockHash,
        uint256 currentTarget,
        uint32 lastPeriodStartTime,
        uint32 lastPeriodEndTime,
        bytes calldata headers
    ) internal pure returns (uint256[] memory, bytes32) {
        bytes32 parentBlockHash = ancestorBlockHash;

        uint256 numHeaders = ProofDecoder.size(headers);
        uint256 k = (numHeaders - 1) / 2016 + 1;

        uint256[] memory retargets = new uint256[](k);

        for (uint256 i = 0; i < k; i++) {
            uint32 periodStartTime = i == 0 ? lastPeriodStartTime : ProofDecoder.get(headers, (i - 1) * 2016).timestamp();
            uint32 periodEndTime = i == 0 ? lastPeriodEndTime : ProofDecoder.get(headers, i * 2016 - 1).timestamp();
            uint256 nextTarget = _adjustTarget(i == 0 ? currentTarget : retargets[i - 1], periodStartTime, periodEndTime);

            retargets[i] = _nBitsToTarget(ProofDecoder.get(headers, i * 2016).nBits()) & nextTarget;

            parentBlockHash = _verify(parentBlockHash, nextTarget, ProofDecoder.slice(headers, i * 2016, i < k - 1 ? (i + 1) * 2016 : numHeaders));
        }

        return (retargets, parentBlockHash);
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

        if (nWord != 0 && ((nBits & 0x00800000) != 0 || ((nSize > 34) || (nWord > 0xff && nSize > 33) || (nWord > 0xffff && nSize > 32)))) {
            revert InvalidTarget();
        }
    }

    function _adjustTarget(uint256 target, uint32 periodStartTime, uint32 periodEndTime) internal pure returns (uint256 newTarget) {
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
