// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title ILightClient Interface
/// @dev Interface for the BTCX Light Client contract which handles Bitcoin block header verification and chain management within EVM-like networks.
interface ILightClient {
    error InvalidGenesisBlockHeight();
    error InvalidSubmitter();
    error BlockNotFound();
    error ForksNotSupported();

    /// @notice Event emitted when a new block is added as the new tip of the blockchain.
    /// @param height The height of the new tip block.
    /// @param hash The hash of the new tip block.
    event NewTip(uint64 indexed height, bytes32 indexed hash);

    /// @notice Retrieves the height of the most recent block in the chain.
    /// @return The height of the latest block in the blockchain.
    function bestBlockHeight() external view returns (uint64);

    /// @notice Gets the hash of the most recent block in the chain.
    /// @return The hash of the latest block.
    function bestBlockHash() external view returns (bytes32);

    /// @notice Fetches the block hash by its height.
    /// @dev Reverts if the block at the given height is not found.
    /// @param height The height of the block to retrieve.
    /// @return The hash of the block at the specified height.
    function blockByHeight(uint64 height) external view returns (bytes32);

    /// @notice Returns the block height for a given block hash.
    /// @dev Reverts if the block with the given hash is not found.
    /// @param blockHash The hash of the block to find.
    /// @return The height of the block with the specified hash.
    function blockByHash(bytes32 blockHash) external view returns (uint64);

    /// @notice Checks if a block has achieved a sufficient number of confirmations.
    /// @dev Uses a predefined constant for the minimum number of confirmations required.
    /// @param blockHash The hash of the block to check.
    /// @return True if the block is confirmed, false otherwise.
    function blockConfirmed(bytes32 blockHash) external view returns (bool);

    /// @notice Retrieves the target difficulty for a given block height.
    /// @dev Reverts if the block at the given height is not found.
    /// @param height The height of the block to retrieve the target for.
    /// @return The target difficulty for the block at the specified height.
    function targetByHeight(uint64 height) external view returns (uint256);

    /// @notice Submits a new block sequence to the chain.
    /// @dev Applies the new blocks to the chain and updates the tip.
    /// @param parentBlockHash The hash of the parent block.
    /// @param blockHashes The hashes of the blocks to submit.
    function submit(bytes32 parentBlockHash, bytes32[] memory blockHashes) external;

    /// @notice Submits a block sequence to the chain for a new period.
    /// @dev Applies the new blocks to the chain with retargeting and updates the tip.
    /// @param parentBlockHash The hash of the parent block.
    /// @param blockHashes The hashes of the blocks to submit.
    /// @param nextTarget The retargeted difficulty for the next block period.
    function submit(bytes32 parentBlockHash, bytes32[] memory blockHashes, uint256 nextTarget) external;
}
