// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IBlockVerifier {
    error BlockVerificationNotSupported();
    error RetargetingNotSupported();

    error InvalidHeadersInput();

    error BlockHashMismatch();
    error InvalidParentHash();

    error InvalidTarget();
    error WorkBelowTarget();
    error NotSamePeriod();

    error InvalidStartPeriodBlockHash();
    error InvalidEndPeriodBlockHash();

    /**
     * @dev Verifies a series of Bitcoin block headers against a given proof.
     * @param ancestorBlockHash The hash of the ancestor block.
     * @param currentTarget The current target.
     * @param headers The headers sequence.
     */
    function verify(
        bytes32 ancestorBlockHash,
        uint256 currentTarget,
        bytes calldata headers
    ) external pure returns (bytes32[] memory);

    /**
     * @dev Verifies a series of Bitcoin block headers with retargeting against a given proof.
     * @param ancestorBlockHeight The height of the ancestor block.
     * @param ancestorBlockHash The hash of the ancestor block.
     * @param startPeriodBlockHash The hash of the start period block.
     * @param currentTarget The current target difficulty.
     * @param headers The headers sequence.
     * @return The array of adjusted targets after retargeting.
     */
    function verifyWithRetargeting(
        uint64 ancestorBlockHeight,
        bytes32 ancestorBlockHash,
        bytes32 startPeriodBlockHash,
        uint256 currentTarget,
        bytes calldata headers
    ) external pure returns (bytes32[] memory, uint256);
}
