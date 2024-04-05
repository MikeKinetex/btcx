// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IVerifier} from "../../interfaces/IVerifier.sol";
import {ProofDecoder} from "./lib/ProofDecoder.sol";
import {BitcoinHeader} from "./lib/BitcoinHeader.sol";

contract EVMVerifier is IVerifier {
    using ProofDecoder for bytes;
    using BitcoinHeader for bytes;

    function verify(
        bytes32[] calldata blockHashes,
        uint64 /* ancestorBlockHeight */,
        bytes32 ancestorBlockHash,
        uint256 currentTarget,
        bytes calldata proof
    ) external pure {
        _verify(blockHashes, ancestorBlockHash, currentTarget, proof);
    }

    function verifyWithRetargeting(
        bytes32[] calldata blockHashes,
        uint64 ancestorBlockHeight,
        bytes32 ancestorBlockHash,
        bytes32 startPeriodBlockHash,
        uint256 currentTarget,
        bytes calldata proof
    ) external pure returns (uint256[] memory retargets) {
        return _verifyWithRetargeting(blockHashes, ancestorBlockHeight, ancestorBlockHash, startPeriodBlockHash, currentTarget, proof);
    }

    function verifyUtreexo(bytes32 /* blockHash */, bytes32 /* parentUtreexo */, bytes calldata /* proof */) external pure returns (bytes32[] memory) {
        revert UtreexoNotSupported();
    }

    function _verify(bytes32[] calldata blockHashes, bytes32 ancestorBlockHash, uint256 currentTarget, bytes calldata proof) internal pure {
        if (blockHashes.length != proof.size()) revert InvalidProofLength();

        bytes32 parentHash = ancestorBlockHash;

        for (uint256 i = 0; i < blockHashes.length; i++) {
            bytes32 blockHash = blockHashes[i];
            bytes calldata header = proof.getHeader(i);

            if (header.hash() != blockHash) revert BlockHashMismatch();
            if (header.parentHash() != parentHash) revert InvalidParentHash();

            uint256 target = _nBitsToTarget(header.nBits());
            if ((target & currentTarget) != target) revert NotSamePeriod();
            if (uint256(blockHash) <= target) revert WorkBelowTarget();

            parentHash = blockHash;
        }
    }

    function _verifyWithRetargeting(
        bytes32[] calldata blockHashes,
        uint64 ancestorBlockHeight,
        bytes32 ancestorBlockHash,
        bytes32 startPeriodBlockHash,
        uint256 currentTarget,
        bytes calldata proof
    ) internal pure returns (uint256[] memory retargets) {
        uint64 delta = (2016 - ((ancestorBlockHeight + 1) % 2016)) % 2016;
        bytes calldata headers = delta > 0 ? proof[80:] : proof[80 * 2:];

        if (proof.getHeader(0).hash() != startPeriodBlockHash) {
            revert InvalidStartPeriodBlockHash();
        }

        if (delta > 0) {
            _verify(blockHashes[:delta], ancestorBlockHash, currentTarget, headers[:80 * (delta + 1)]);
        } else {
            if (proof.getHeader(1).hash() != ancestorBlockHash) {
                revert InvalidEndPeriodBlockHash();
            }
        }

        uint64 k = (uint64(blockHashes.length) - delta - 1) / 2016 + 1;

        for (uint64 i = 0; i < k; i++) {
            uint256 startIndex = delta + i * 2016;
            uint256 endIndex = i < k - 1 ? delta + (i + 1) * 2016 : blockHashes.length;

            uint32 periodStartTime = i == 0 && delta == 0 ? proof.getHeader(0).timestamp() : headers.getHeader(startIndex).timestamp();
            uint32 periodEndTime = i == 0 && delta == 0 ? proof.getHeader(1).timestamp() : headers.getHeader(endIndex).timestamp();

            bytes32 parentBlockHash = i == 0 && delta == 0 ? ancestorBlockHash : blockHashes[startIndex - 1];
            uint256 target = _adjustTarget(i == 0 ? currentTarget : retargets[i - 1], periodStartTime, periodEndTime);

            _verify(blockHashes[startIndex:endIndex], parentBlockHash, target, headers[80 * startIndex:80 * endIndex]);

            retargets[i] = _nBitsToTarget(headers.getHeader(startIndex).nBits()) & target;
        }
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
