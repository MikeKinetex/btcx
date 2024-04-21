// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ILightClient} from "./interfaces/ILightClient.sol";

contract BTCX is ILightClient {
    uint256 public constant MIN_CONFIRMATIONS = 6;

    uint64 public immutable GENESIS_BLOCK_HEIGHT;
    bytes32 public immutable GENESIS_BLOCK_HASH;

    uint64 internal chainTip;
    uint256 internal chainWork;

    mapping(uint64 => bytes32) internal hashes;
    mapping(bytes32 => uint64) internal blocks;
    mapping(bytes32 => bytes32) internal utreexo;
    mapping(uint64 => uint256) internal retargets;

    mapping(address => bool) public allowedSubmitters;

    modifier validateByHeight(uint64 height) {
        if (height > chainTip || height < GENESIS_BLOCK_HEIGHT) revert BlockNotFound();
        _;
    }

    modifier validateByHash(bytes32 blockHash) {
        if (blocks[blockHash] == 0 && blockHash != GENESIS_BLOCK_HASH) revert BlockNotFound();
        _;
    }

    modifier onlySubmitter() {
        if (!allowedSubmitters[msg.sender]) revert InvalidSubmitter();
        _;
    }

    constructor(
        uint64 genesisBlockHeight,
        bytes32 genesisBlockHash,
        bytes32 genesisBlockUtreexo,
        uint256 genesisBlockTarget,
        address[] memory submitters
    ) {
        if (genesisBlockHeight % 2016 != 0) revert InvalidGenesisBlockHeight();

        GENESIS_BLOCK_HEIGHT = genesisBlockHeight;
        GENESIS_BLOCK_HASH = genesisBlockHash;

        chainTip = genesisBlockHeight;
        chainWork = _calculateChainWork(genesisBlockTarget);

        hashes[genesisBlockHeight] = genesisBlockHash;
        blocks[genesisBlockHash] = genesisBlockHeight;
        utreexo[genesisBlockHash] = genesisBlockUtreexo;
        retargets[genesisBlockHeight / 2016] = genesisBlockTarget;

        for (uint256 i = 0; i < submitters.length; i++) {
            allowedSubmitters[submitters[i]] = true;
        }
    }

    function bestBlockHeight() external view returns (uint64) {
        return chainTip;
    }

    function bestBlockHash() external view returns (bytes32) {
        return hashes[chainTip];
    }

    function blockByHeight(uint64 height) external view validateByHeight(height) returns (bytes32) {
        return hashes[height];
    }

    function blockByHash(bytes32 blockHash) external view validateByHash(blockHash) returns (uint64) {
        return blocks[blockHash];
    }

    function blockConfirmed(bytes32 blockHash) external view validateByHash(blockHash) returns (bool) {
        return blocks[blockHash] + MIN_CONFIRMATIONS <= chainTip;
    }

    function targetByHeight(uint64 height) external view validateByHeight(height) returns (uint256) {
        return retargets[height / 2016];
    }

    function submit(
        bytes32 parentBlockHash,
        bytes32[] memory blockHashes
    ) external validateByHash(parentBlockHash) onlySubmitter {
        uint64 parentBlockHeight = blocks[parentBlockHash];

        // ignore forks with less work done
        if (parentBlockHeight + blockHashes.length <= chainTip) revert ForksNotSupported();

        // calculate chainWork
        uint256 currentTarget = retargets[parentBlockHeight / 2016];
        chainWork +=
            _calculateChainWork(currentTarget) *
            (parentBlockHeight == chainTip ? blockHashes.length : (blockHashes.length + parentBlockHeight - chainTip));

        // update the chain with new blocks
        _updateChainWithNewBlocks(parentBlockHeight, blockHashes);
    }

    function submit(
        bytes32 parentBlockHash,
        bytes32[] memory blockHashes,
        uint256 nextTarget
    ) external validateByHash(parentBlockHash) onlySubmitter {
        uint64 parentBlockHeight = blocks[parentBlockHash];
        uint256 currentTarget = retargets[parentBlockHeight / 2016];
        uint64 startPeriodBlockHeight = parentBlockHeight - (parentBlockHeight % 2016);
        uint64 endPeriodBlockHeight = startPeriodBlockHeight + 2015;

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
                chainWork_ += _calculateChainWork(nextTarget);
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

        // prune chain if needed
        if (parentBlockHeight != chainTip) {
            for (uint64 i = chainTip; i > parentBlockHeight; i--) {
                delete hashes[i];
                if (i % 2016 == 0) {
                    delete retargets[i / 2016];
                }
            }
        }

        // update retargets
        if (nextTarget > 0) {
            retargets[startPeriodBlockHeight / 2016 + 1] = nextTarget;
        }

        // update the chain with new blocks
        _updateChainWithNewBlocks(parentBlockHeight, blockHashes);
    }

    function _calculateChainWork(uint256 target) internal pure returns (uint256) {
        return (~target / (target + 1)) + 1;
    }

    function _updateChainWithNewBlocks(uint64 parentBlockHeight, bytes32[] memory blockHashes) internal {
        uint64 chainTip_ = parentBlockHeight;

        for (uint256 i = 0; i < blockHashes.length; i++) {
            hashes[++chainTip_] = blockHashes[i];
            blocks[blockHashes[i]] = chainTip_;
        }

        chainTip = chainTip_;

        emit NewTip(chainTip, hashes[chainTip_]);
    }
}
