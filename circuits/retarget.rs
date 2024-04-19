use ethers::types::U256;
use plonky2x::backend::circuit::Circuit;
use plonky2x::frontend::hint::simple::hint::Hint;
use plonky2x::frontend::vars::U256Variable;
use plonky2x::prelude::{
    ArrayVariable, CircuitBuilder, PlonkParameters, U64Variable, ValueStream, VariableStream,
};

use serde::{Deserialize, Serialize};

use crate::builder::multi::BitcoinMultiVerify;
use crate::input::InputDataFetcher;
use crate::vars::*;

pub trait BitcoinVerifyWithRetargetCircuit<L: PlonkParameters<D>, const D: usize> {
    fn verify_with_retargeting<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: U64Variable,
        prev_header_hash: BlockHashVariable,
        period_start_hash: BlockHashVariable,
        current_threshold: ThresholdVariable,
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, ThresholdVariable);
}

impl<L: PlonkParameters<D>, const D: usize> BitcoinVerifyWithRetargetCircuit<L, D>
    for CircuitBuilder<L, D>
{
    fn verify_with_retargeting<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: U64Variable,
        prev_header_hash: BlockHashVariable,
        period_start_hash: BlockHashVariable,
        current_threshold: ThresholdVariable,
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, ThresholdVariable) {
        let mut input_stream = VariableStream::new();
        input_stream.write(&prev_block_number);
        input_stream.write(&prev_header_hash);
        let output_stream = self.hint(
            input_stream,
            VerifyOffchainInputs::<UPDATE_HEADERS_COUNT> {},
        );

        let next_threshold = output_stream.read::<ThresholdVariable>(self);
        let period_start_header_bytes = output_stream.read::<HeaderBytesVariable>(self);
        let period_end_header_bytes = output_stream.read::<HeaderBytesVariable>(self);
        let update_headers_bytes =
            output_stream.read::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(self);

        self.validate_headers_with_retargeting(
            &prev_block_number,
            &prev_header_hash,
            &period_start_hash,
            &current_threshold,
            &next_threshold,
            &period_start_header_bytes,
            &period_end_header_bytes,
            &update_headers_bytes,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOffchainInputs<const UPDATE_HEADERS_COUNT: usize> {}

// #[async_trait]
impl<const UPDATE_HEADERS_COUNT: usize, L: PlonkParameters<D>, const D: usize> Hint<L, D>
    for VerifyOffchainInputs<UPDATE_HEADERS_COUNT>
{
    fn hint(&self, input_stream: &mut ValueStream<L, D>, output_stream: &mut ValueStream<L, D>) {
        let prev_block_number = input_stream.read_value::<U64Variable>();
        let prev_header_hash = input_stream.read_value::<BlockHashVariable>();

        let mut input_fetcher = InputDataFetcher::default();

        let period_start_block_number = prev_block_number - prev_block_number % 2016;
        let period_end_block_number = period_start_block_number + 2015;

        let period_start_header = input_fetcher.get_header_by_height(period_start_block_number);
        let period_start_header_bytes = input_fetcher.to_bytes(&period_start_header);

        let period_end_header = input_fetcher.get_header_by_height(period_end_block_number);
        let period_end_header_bytes = input_fetcher.to_bytes(&period_end_header);

        let next_period_start_header =
            input_fetcher.get_header_by_height(period_end_block_number + 1);
        let next_threshold =
            U256::from_little_endian(&next_period_start_header.target().to_le_bytes());

        let update_headers_bytes = input_fetcher
            .get_update_headers_inputs::<UPDATE_HEADERS_COUNT>(prev_header_hash);

        output_stream.write_value::<ThresholdVariable>(next_threshold);
        output_stream.write_value::<HeaderBytesVariable>(period_start_header_bytes);
        output_stream.write_value::<HeaderBytesVariable>(period_end_header_bytes);
        output_stream.write_value::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(
            update_headers_bytes,
        );
    }
}

#[derive(Debug, Clone)]
pub struct VerifyWithRetargetCircuit<const UPDATE_HEADERS_COUNT: usize> {}

impl<const UPDATE_HEADERS_COUNT: usize> Circuit
    for VerifyWithRetargetCircuit<UPDATE_HEADERS_COUNT>
{
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let prev_block_number = builder.evm_read::<U64Variable>();
        let prev_header_hash = builder.evm_read::<BlockHashVariable>();
        let period_start_hash = builder.evm_read::<BlockHashVariable>();
        let current_threshold = builder.evm_read::<ThresholdVariable>();

        let (header_hashes, next_threshold) = builder
            .verify_with_retargeting::<UPDATE_HEADERS_COUNT>(
                prev_block_number,
                prev_header_hash,
                period_start_hash,
                current_threshold,
            );
        
        header_hashes.as_vec().iter().for_each(|hash| {
            builder.evm_write(*hash);
        });
        builder.evm_write::<U256Variable>(next_threshold);
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

    use ethers::types::H256;
    use plonky2x::prelude::{bytes32, DefaultBuilder, GateRegistry, HintRegistry};

    use super::*;

    #[test]
    fn test_verify_serialization() {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        const UPDATE_HEADERS_COUNT: usize = 2;
        let mut builder = DefaultBuilder::new();

        log::debug!("Defining circuit");
        VerifyWithRetargetCircuit::<UPDATE_HEADERS_COUNT>::define(&mut builder);
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut hint_registry = HintRegistry::new();
        let mut gate_registry = GateRegistry::new();
        VerifyWithRetargetCircuit::<UPDATE_HEADERS_COUNT>::register_generators(&mut hint_registry);
        VerifyWithRetargetCircuit::<UPDATE_HEADERS_COUNT>::register_gates(&mut gate_registry);

        circuit.test_serializers(&gate_registry, &hint_registry);
    }

    fn test_verify_with_retargeting_template<const UPDATE_HEADERS_COUNT: usize>(
        prev_block_number: u64,
        prev_header_hash: H256,
        period_start_hash: H256,
        current_threshold: U256,
    ) {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        let mut builder = DefaultBuilder::new();

        log::debug!("Defining circuit");
        VerifyWithRetargetCircuit::<UPDATE_HEADERS_COUNT>::define(&mut builder);

        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut input = circuit.input();
        input.evm_write::<U64Variable>(prev_block_number);
        input.evm_write::<BlockHashVariable>(prev_header_hash);
        input.evm_write::<BlockHashVariable>(period_start_hash);
        input.evm_write::<ThresholdVariable>(current_threshold);

        log::debug!("Generating proof");
        let (proof, mut output) = circuit.prove(&input);
        log::debug!("Done generating proof");

        circuit.verify(&proof, &input, &output);

        let next_threshold = output.evm_read::<ThresholdVariable>();
        log::debug!("next_threshold {:?}", next_threshold);

        for i in 0..UPDATE_HEADERS_COUNT {
            let hash = output.evm_read::<BlockHashVariable>();
            log::debug!("header hash {}: {}", i, hash);
        }
    }

    #[test]
    fn test_verify_with_retargeting_203610_10() {
        const UPDATE_HEADERS_COUNT: usize = 10;
        let prev_block_number = 203610;
        let prev_header_hash =
            bytes32!("a12e1f2157c6f99469ccdb46ae68577273d3551f6a38d17ab304000000000000");
        let period_start_hash =
            bytes32!("d09acdf9c9959a1754da9dae916e70bef9f131ad30ef8be2a503000000000000");
        let current_threshold =
            U256::from_dec_str("8825801199382903987726989797449454220615414953524072026210304")
                .unwrap();
        test_verify_with_retargeting_template::<UPDATE_HEADERS_COUNT>(
            prev_block_number,
            prev_header_hash,
            period_start_hash,
            current_threshold,
        );
    }

    #[test]
    fn test_verify_with_retargeting_2015_10() {
        const UPDATE_HEADERS_COUNT: usize = 10;
        let prev_block_number = 2015;
        let prev_header_hash =
            bytes32!("6397bb6abd4fc521c0d3f6071b5650389f0b4551bc40b4e6b067306900000000");
        let period_start_hash =
            bytes32!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000");
        let current_threshold = U256::from_dec_str(
            "26959535291011309493156476344723991336010898738574164086137773096960",
        )
        .unwrap();
        test_verify_with_retargeting_template::<UPDATE_HEADERS_COUNT>(
            prev_block_number,
            prev_header_hash,
            period_start_hash,
            current_threshold,
        );
    }

    #[test]
    fn test_verify_with_retargeting_2016_10() {
        const UPDATE_HEADERS_COUNT: usize = 10;
        let prev_block_number = 2016;
        let prev_header_hash =
            bytes32!("efdd7b6c4ce1dcbb370690558d7a556e431c3011f2546c896a2141a100000000");
        let period_start_hash =
            bytes32!("efdd7b6c4ce1dcbb370690558d7a556e431c3011f2546c896a2141a100000000");
        let current_threshold = U256::from_dec_str(
            "26959535291011309493156476344723991336010898738574164086137773096960",
        )
        .unwrap();
        test_verify_with_retargeting_template::<UPDATE_HEADERS_COUNT>(
            prev_block_number,
            prev_header_hash,
            period_start_hash,
            current_threshold,
        );
    }
}
