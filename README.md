# BTCX

BTCX is a Bitcoin ZK light client, which offers the capability to verify Bitcoin block headers on EVM-compatible networks. Its implementation enables the validation of completed transactions on the Bitcoin network, establishing a high-security level of interoperability between the two largest ecosystems. The project is built with the plonky2x framework.

### Overview

BTCX contains a set of ZK circuits that can be used to implement a Bitcoin light client. An appropriate smart-contract can be implemented to validate the header of a specific block height or the latest header. The primary circuit `verify` serves as the entry point.

**verify**

The `verify` circuit validates a sequence of Bitcoin headers.

Taking `prev_header_hash` as input along with a sequence of headers bytes, this circuit outputs a proof containing corresponding hashes and the total work for the provided headers if they are valid.

The Bitcoin block header verification algorithm ensures that the hash matches the block header, the header's work is within the difficulty bits, and the parent hash of the current block matches the previous header's hash.

The circuit employs a STARK-based accelerator, built with the curta library, to optimize SHA256 computations and reduce proving time.

### Initial setup and updates

Due to the nature of the [PoW](https://en.bitcoin.it/wiki/Proof_of_work) consensus mechanism, there are no validators selecting the exclusive correct chain. Therefore, in the context of Bitcoin, the correct chain is the “longest chain”, indicating it has the highest cumulative work. Consequently, the prior chain may be partially or entirely pruned and replaced if the newly provided chain has a greater amount of chainwork.

The provided circuit enables the validation of any chain of blocks without requiring specific context or information about the network and time period to which the sequence belongs.

Therefore, it is very important to initiate the light client correctly by setting the initial, correct genesis block or verified checkpoint, ensuring its immutability.

Additionally, it is assumed that the light client should record the accumulated work and handle block reorganization. Given that any sequence can be deemed correct if it aligns with a previously saved chain and adheres to the rules of the Bitcoin network, the light client should follow incoming updates by promptly integrating them with the correct blockchain. This process should override any other previous chains with less work done, while relying on appropriately set parameters for finalizing the chain.

It is important to follow the full set of Bitcoin consensus rules. A key aspect of ensuring the correctness of the chain involves confirming alignment with the [difficulty adjustment](https://en.bitcoin.it/wiki/Difficulty#What_network_hash_rate_results_in_a_given_difficulty.3F) formula. Every 2016 blocks, the current difficulty undergoes recalculation based on the time spent to find the previous 2016 blocks. Additionally, all blocks within the same window should share the same target, and the blocks of the subsequent period should have a target corresponding to the correct adjustment. The implementation of this validation process is planned for the near future.

### Deployment

The circuits are available on Succinct X [here](https://alpha.succinct.xyz/@MikeKinetex/btcx).

### Further improvements

- Implement a check for retargeting (for every 2016 blocks);
- Possibly revising the calculation of work performed;
- Implement and deploy a light client smart-contract;
- Include support for MMR (Merkle Mountain Range).

### Credits

This project is based on [btc-warp](https://github.com/succinctlabs/btc-warp), originally implemented in plonky2, with the aim of achieving a much faster synchronization of Bitcoin nodes.