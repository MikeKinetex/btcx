use plonky2x::backend::circuit::Circuit;
use plonky2x::frontend::hint::simple::hint::Hint;
use plonky2x::prelude::{
    ArrayVariable, CircuitBuilder, PlonkParameters, ValueStream, VariableStream,
};

use serde::{Deserialize, Serialize};

use crate::builder::multi::BitcoinMultiVerify;
use crate::input::InputDataFetcher;
use crate::vars::*;

pub trait BitcoinVerifyCircuit<L: PlonkParameters<D>, const D: usize> {
    fn verify<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_header_hash: BlockHashVariable,
        threshold: ThresholdVariable,
    ) -> ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>;
}

impl<L: PlonkParameters<D>, const D: usize> BitcoinVerifyCircuit<L, D> for CircuitBuilder<L, D> {
    fn verify<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_header_hash: BlockHashVariable,
        threshold: ThresholdVariable,
    ) -> ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT> {
        let mut input_stream = VariableStream::new();
        input_stream.write(&prev_header_hash);
        let output_stream = self.hint(
            input_stream,
            VerifyOffchainInputs::<UPDATE_HEADERS_COUNT> {},
        );
        let update_headers_bytes =
            output_stream.read::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(self);

        self.validate_headers(&prev_header_hash, &threshold, &update_headers_bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOffchainInputs<const UPDATE_HEADERS_COUNT: usize> {}

// #[async_trait]
impl<const UPDATE_HEADERS_COUNT: usize, L: PlonkParameters<D>, const D: usize> Hint<L, D>
    for VerifyOffchainInputs<UPDATE_HEADERS_COUNT>
{
    fn hint(&self, input_stream: &mut ValueStream<L, D>, output_stream: &mut ValueStream<L, D>) {
        let prev_header_hash = input_stream.read_value::<BlockHashVariable>();

        let mut input_fetcher = InputDataFetcher::default();
        let update_headers_bytes = input_fetcher
            .get_update_headers_inputs::<UPDATE_HEADERS_COUNT>(prev_header_hash);

        output_stream.write_value::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(
            update_headers_bytes
        );
    }
}

#[derive(Debug, Clone)]
pub struct VerifyCircuit<const UPDATE_HEADERS_COUNT: usize> {}

impl<const UPDATE_HEADERS_COUNT: usize> Circuit for VerifyCircuit<UPDATE_HEADERS_COUNT> {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let prev_header_hash = builder.evm_read::<BlockHashVariable>();
        let threshold = builder.evm_read::<ThresholdVariable>();

        let header_hashes =
            builder.verify::<UPDATE_HEADERS_COUNT>(prev_header_hash, threshold);

        header_hashes.as_vec().iter().for_each(|hash| {
            builder.evm_write(*hash);
        });
    }

    fn register_generators<L: PlonkParameters<D>, const D: usize>(
        generator_registry: &mut plonky2x::prelude::HintRegistry<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<L::Field>,
    {
        generator_registry.register_hint::<VerifyOffchainInputs<UPDATE_HEADERS_COUNT>>();
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use ethers::types::{H256, U256};
    use plonky2x::prelude::{bytes32, DefaultBuilder, GateRegistry, HintRegistry};

    use super::*;

    #[test]
    fn test_verify_serialization() {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        const UPDATE_HEADERS_COUNT: usize = 2;
        let mut builder = DefaultBuilder::new();

        log::debug!("Defining circuit");
        VerifyCircuit::<UPDATE_HEADERS_COUNT>::define(&mut builder);
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut hint_registry = HintRegistry::new();
        let mut gate_registry = GateRegistry::new();
        VerifyCircuit::<UPDATE_HEADERS_COUNT>::register_generators(&mut hint_registry);
        VerifyCircuit::<UPDATE_HEADERS_COUNT>::register_gates(&mut gate_registry);

        circuit.test_serializers(&gate_registry, &hint_registry);
    }

    fn test_verify_template<const UPDATE_HEADERS_COUNT: usize>(
        prev_header_hash: H256,
        threshold: U256,
    ) -> Vec<H256> {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        let mut builder = DefaultBuilder::new();

        log::debug!("Defining circuit");
        VerifyCircuit::<UPDATE_HEADERS_COUNT>::define(&mut builder);

        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut input = circuit.input();
        input.evm_write::<BlockHashVariable>(prev_header_hash);
        input.evm_write::<ThresholdVariable>(threshold);

        log::debug!("Generating proof");
        let (proof, mut output) = circuit.prove(&input);
        log::debug!("Done generating proof");

        circuit.verify(&proof, &input, &output);

        let mut hashes = Vec::new();
        for i in 0..UPDATE_HEADERS_COUNT {
            let hash = output.evm_read::<BlockHashVariable>();
            log::debug!("header hash {}: {}", i, hash);
            hashes.push(hash);
        }

        return hashes;
    }

    #[test]
    fn test_verify_genesis_10() {
        const UPDATE_HEADERS_COUNT: usize = 10;
        let header = bytes32!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000");
        let threshold = U256::from_dec_str(
            "26959535291011309493156476344723991336010898738574164086137773096960",
        )
        .unwrap();
        let mut hashes = test_verify_template::<UPDATE_HEADERS_COUNT>(header, threshold);
        assert_eq!(hashes.len(), UPDATE_HEADERS_COUNT);
        assert_eq!(hashes.pop().unwrap(), bytes32!("e915d9a478e3adf3186c07c61a22228b10fd87df343c92782ecc052c00000000"));
    }

    #[test]
    fn test_verify_200000_100() {
        const UPDATE_HEADERS_COUNT: usize = 100;
        let header = bytes32!("bf0e2e13fce62f3a5f15903a177ad6a258a01f164aefed7d4a03000000000000");
        let threshold =
            U256::from_dec_str("9412783771427520201810837309176674245361798887059324066070528")
                .unwrap();
        let mut hashes = test_verify_template::<UPDATE_HEADERS_COUNT>(header, threshold);
        assert_eq!(hashes.len(), UPDATE_HEADERS_COUNT);
        assert_eq!(hashes.pop().unwrap(), bytes32!("2a051182bc468e29d8fc925550ebac17ccec5bca3eaa107f5d04000000000000"));
    }
}
