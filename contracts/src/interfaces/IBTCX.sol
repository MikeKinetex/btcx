// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title IBTCX Interface
/// @dev Interface for the BTCX contract which handles Bitcoin block header verification and chain management within EVM-like networks.
interface IBTCX {
    error InvalidGenesisBlockHeight();
    error InvalidInput();
    error BlockNotFound();
    error UtreexoNotFound();
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

    /// @notice Submits a series of block hashes for verification and potentially extends the blockchain.
    /// @dev This function may trigger a retargeting process depending on the blocks' heights and the current blockchain state.
    /// @param blockHashes An array of new block hashes to submit.
    /// @param parentBlockHash The hash of the parent block to the first block in the `blockHashes` array.
    /// @param proof The proof data required for verification of the block hashes.
    function submit(bytes32[] calldata blockHashes, bytes32 parentBlockHash, bytes calldata proof) external;

    /// @notice Submits a Utreexo proof for verification against a specified block hash.
    /// @dev This function updates the Utreexo state for a block, verifying the inclusion or exclusion of UTXOs.
    /// @param blockHash The hash of the block for which the Utreexo proof is submitted.
    /// @param proof The Utreexo proof data required for verification.
    function submitUtreexo(bytes32 blockHash, bytes calldata proof) external;
}
