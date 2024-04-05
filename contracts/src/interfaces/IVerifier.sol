// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IVerifier {
    error BlockVerificationNotSupported();
    error RetargetingNotSupported();
    error UtreexoNotSupported();

    error InvalidProofLength();

    error BlockHashMismatch();
    error InvalidParentHash();

    error InvalidTarget();
    error WorkBelowTarget();
    error NotSamePeriod();

    error InvalidStartPeriodBlockHash();
    error InvalidEndPeriodBlockHash();

    /**
     * @dev Verifies a series of Bitcoin block headers against a given proof.
     * @param blockHashes The array of block hashes to verify.
     * @param ancestorBlockHeight The height of the ancestor block.
     * @param ancestorBlockHash The hash of the ancestor block.
     * @param currentTarget The current target.
     * @param proof The proof data.
     */
    function verify(
        bytes32[] calldata blockHashes,
        uint64 ancestorBlockHeight,
        bytes32 ancestorBlockHash,
        uint256 currentTarget,
        bytes calldata proof
    ) external view;

    /**
     * @dev Verifies a series of Bitcoin block headers with retargeting against a given proof.
     * @param blockHashes The array of block hashes to verify.
     * @param ancestorBlockHeight The height of the ancestor block.
     * @param ancestorBlockHash The hash of the ancestor block.
     * @param startPeriodBlockHash The hash of the start period block.
     * @param currentTarget The current target difficulty.
     * @param proof The proof data.
     * @return The array of adjusted targets after retargeting.
     */
    function verifyWithRetargeting(
        bytes32[] calldata blockHashes,
        uint64 ancestorBlockHeight,
        bytes32 ancestorBlockHash,
        bytes32 startPeriodBlockHash,
        uint256 currentTarget,
        bytes calldata proof
    ) external view returns (uint256[] memory);

    /**
     * @dev Updates and verifies Utreexo against a given proof.
     * @param blockHash The hash of the block for which Utreexo needs to be updated and verified.
     * @param parentUtreexo The commitment of previous Utreexo roots.
     * @param proof The Utreexo proof data.
     * @return The updated Utreexo roots after verification.
     */
    function verifyUtreexo(bytes32 blockHash, bytes32 parentUtreexo, bytes calldata proof) external view returns (bytes32[] memory);
}
