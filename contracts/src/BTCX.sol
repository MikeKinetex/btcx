// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IBTCX} from "./interfaces/IBTCX.sol";
import {IVerifier} from "./interfaces/IVerifier.sol";

contract BTCX is IBTCX {
    uint256 public constant MIN_CONFIRMATIONS = 6;
    IVerifier public immutable VERIFIER;

    uint64 public immutable GENESIS_BLOCK_HEIGHT;
    bytes32 public immutable GENESIS_BLOCK_HASH;

    uint64 private chainTip;
    uint256 private chainWork;

    mapping(uint64 => bytes32) private hashes;
    mapping(bytes32 => uint64) private blocks;
    mapping(bytes32 => bytes32) private utreexo;
    mapping(uint64 => uint256) private retargets;

    constructor(
        uint64 genesisBlockHeight,
        bytes32 genesisBlockHash,
        bytes32 genesisBlockUtreexo,
        uint256 genesisBlockTarget,
        address verifier
    ) {
        if (genesisBlockHeight % 2016 != 0) revert InvalidGenesisBlockHeight();

        GENESIS_BLOCK_HEIGHT = genesisBlockHeight;
        GENESIS_BLOCK_HASH = genesisBlockHash;
        VERIFIER = IVerifier(verifier);

        chainTip = genesisBlockHeight;
        chainWork = _calculateChainWork(genesisBlockTarget);

        hashes[genesisBlockHeight] = genesisBlockHash;
        blocks[genesisBlockHash] = genesisBlockHeight;
        utreexo[genesisBlockHash] = genesisBlockUtreexo;
        retargets[genesisBlockHeight / 2016] = genesisBlockTarget;
    }

    function bestBlockHeight() external view returns (uint64) {
        return chainTip;
    }

    function bestBlockHash() external view returns (bytes32) {
        return hashes[chainTip];
    }

    function blockByHeight(uint64 height) external view returns (bytes32) {
        _validateByHeight(height);
        return hashes[height];
    }

    function blockByHash(bytes32 blockHash) external view returns (uint64) {
        _validateByHash(blockHash);
        return blocks[blockHash];
    }

    function blockConfirmed(bytes32 blockHash) external view returns (bool) {
        _validateByHash(blockHash);
        return blocks[blockHash] + MIN_CONFIRMATIONS <= chainTip;
    }

    function submit(bytes32[] calldata blockHashes, bytes32 parentBlockHash, bytes calldata proof) external {
        _validateByHash(parentBlockHash);

        uint64 parentBlockHeight = blocks[parentBlockHash];
        uint256 currentTarget = retargets[parentBlockHeight / 2016];
        uint64 startPeriodBlockHeight = parentBlockHeight - (parentBlockHeight % 2016);
        uint64 endPeriodBlockHeight = startPeriodBlockHeight + 2015;

        if (parentBlockHeight + blockHashes.length > endPeriodBlockHeight) {
            // verify blocks with retargeting
            bytes32 startPeriodBlockHash = hashes[startPeriodBlockHeight];
            (uint256[] memory newRetargets, bytes32 endBlockHash) = VERIFIER.verifyWithRetargeting(
                parentBlockHeight,
                parentBlockHash,
                startPeriodBlockHash,
                currentTarget,
                proof
            );

            // check end block hash
            if (blockHashes[blockHashes.length - 1] != endBlockHash) {
                revert InvalidInput();
            }

            // calculate chain work (should be optimized)
            uint256 chainWork_ = chainWork;
            if (parentBlockHeight < chainTip) {
                for (uint64 i = chainTip; i > parentBlockHeight; i--) {
                    chainWork_ -= _calculateChainWork(retargets[i / 2016]);
                }
            }

            uint64 newEpochIndex = 0;
            uint64 heightIndex = parentBlockHeight + 1;
            for (uint64 i = 0; i < blockHashes.length; i++) {
                if (heightIndex > endPeriodBlockHeight) {
                    // if the block height is beyond the current retargeting period, calculate work with new target
                    chainWork_ += _calculateChainWork(newRetargets[newEpochIndex]);
                    if (heightIndex % 2016 == 2015) newEpochIndex++;
                } else {
                    chainWork_ += _calculateChainWork(currentTarget);
                }
                heightIndex++;
            }

            // ignore forks with less work done
            if (chainWork_ <= chainWork) revert ForksNotSupported();

            // store new chainwork
            chainWork = chainWork_;

            // prune chain and update retargets if needed
            if (parentBlockHeight != chainTip) {
                for (uint64 i = chainTip; i > parentBlockHeight; i--) {
                    delete hashes[i];
                    if (i % 2016 == 0) {
                        delete retargets[i / 2016];
                    }
                }
            }
            for (uint64 i = 0; i < newRetargets.length; i++) {
                retargets[startPeriodBlockHeight / 2016 + 1 + i] = newRetargets[i];
            }
        } else {
            // ignore forks with less work done
            if (parentBlockHeight + blockHashes.length <= chainTip) revert ForksNotSupported();

            // verify blocks without retargeting
            bytes32 endBlockHash = VERIFIER.verify(parentBlockHeight, parentBlockHash, currentTarget, proof);

            // check end block hash
            if (blockHashes[blockHashes.length - 1] != endBlockHash) {
                revert InvalidInput();
            }

            // calculate chainWork
            chainWork +=
                _calculateChainWork(currentTarget) *
                (
                    parentBlockHeight == chainTip
                        ? blockHashes.length
                        : (blockHashes.length + parentBlockHeight - chainTip)
                );
        }

        // update the chain with new blocks
        _updateChainWithNewBlocks(parentBlockHeight, blockHashes);
    }

    function submitUtreexo(bytes32 blockHash, bytes calldata proof) external {
        _validateByHash(blockHash);

        bytes32 parentUtreexoCommitment = utreexo[hashes[blocks[blockHash] - 1]];
        if (parentUtreexoCommitment == bytes32(0)) revert UtreexoNotFound();

        bytes32[] memory newUtreexoRoots = VERIFIER.verifyUtreexo(blockHash, parentUtreexoCommitment, proof);
        utreexo[blockHash] = keccak256(abi.encode(newUtreexoRoots));
    }

    function _calculateChainWork(uint256 target) internal pure returns (uint256) {
        return (~target / (target + 1)) + 1;
    }

    function _validateByHeight(uint64 height) internal view {
        if (height > chainTip || height < GENESIS_BLOCK_HEIGHT) revert BlockNotFound();
    }

    function _validateByHash(bytes32 blockHash) internal view {
        if (blocks[blockHash] == 0 && blockHash != GENESIS_BLOCK_HASH) revert BlockNotFound();
    }

    function _updateChainWithNewBlocks(uint64 parentBlockHeight, bytes32[] calldata blockHashes) internal {
        uint64 chainTip_ = parentBlockHeight;

        for (uint256 i = 0; i < blockHashes.length; i++) {
            hashes[++chainTip_] = blockHashes[i];
            blocks[blockHashes[i]] = chainTip_;
        }

        chainTip = chainTip_;

        emit NewTip(chainTip, hashes[chainTip_]);
    }
}
