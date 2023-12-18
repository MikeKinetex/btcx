use plonky2x::backend::circuit::Circuit;
use plonky2x::frontend::hint::simple::hint::Hint;
use plonky2x::prelude::{
    ArrayVariable, U64Variable,
    CircuitBuilder, PlonkParameters,
    ValueStream, VariableStream
};

use serde::{Deserialize, Serialize};

use crate::multi::BitcoinMultiVerify;
use crate::input::InputDataFetcher;
use crate::vars::*;

pub trait BitcoinVerifyCircuit<L: PlonkParameters<D>, const D: usize> {
    fn verify<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: U64Variable,
        prev_header_hash: BlockHashVariable,
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, WorkVariable);
}

impl<L: PlonkParameters<D>, const D: usize> BitcoinVerifyCircuit<L, D> for CircuitBuilder<L, D> {
    fn verify<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: U64Variable,
        prev_header_hash: BlockHashVariable,
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, WorkVariable) {
        let mut input_stream = VariableStream::new();
        input_stream.write(&prev_block_number);
        input_stream.write(&prev_header_hash);
        let output_stream = self.hint(
            input_stream,
            VerifyOffchainInputs::<UPDATE_HEADERS_COUNT> {},
        );
        let update_headers_bytes = 
          output_stream.read::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(self);

        self.validate_headers(&prev_header_hash, &update_headers_bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOffchainInputs<const UPDATE_HEADERS_COUNT: usize> {}

// #[async_trait]
impl<const UPDATE_HEADERS_COUNT: usize, L: PlonkParameters<D>, const D: usize> Hint<L, D>
    for VerifyOffchainInputs<UPDATE_HEADERS_COUNT>
{
    fn hint(
        &self,
        input_stream: &mut ValueStream<L, D>,
        output_stream: &mut ValueStream<L, D>,
    ) {
        let prev_block_number = input_stream.read_value::<U64Variable>();
        let prev_header_hash = input_stream.read_value::<BlockHashVariable>();

        let mut input_fetcher = InputDataFetcher::default();
        let update_headers_bytes = input_fetcher.get_update_headers_inputs::<UPDATE_HEADERS_COUNT>(prev_block_number, prev_header_hash);

        output_stream.write_value::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(update_headers_bytes);
    }
}

#[derive(Debug, Clone)]
pub struct VerifyCircuit<const UPDATE_HEADERS_COUNT: usize> {}

impl<const UPDATE_HEADERS_COUNT: usize> Circuit for VerifyCircuit<UPDATE_HEADERS_COUNT> {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let prev_block_number = builder.evm_read::<U64Variable>();
        let prev_header_hash = builder.evm_read::<BlockHashVariable>();

        let (update_headers, total_work) =
            builder.verify::<UPDATE_HEADERS_COUNT>(prev_block_number, prev_header_hash);

        for i in 0..update_headers.len() {
          builder.evm_write(update_headers[i]);
        }
        
        builder.evm_write(total_work);
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
    use plonky2x::prelude::{
        bytes32,
        DefaultBuilder, GateRegistry, HintRegistry
    };

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
        prev_block_number: u64,
        prev_header_hash: H256,
    ) {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        let mut builder = DefaultBuilder::new();

        log::debug!("Defining circuit");
        VerifyCircuit::<UPDATE_HEADERS_COUNT>::define(&mut builder);

        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut input = circuit.input();
        input.evm_write::<U64Variable>(prev_block_number);
        input.evm_write::<BlockHashVariable>(prev_header_hash);

        log::debug!("Generating proof");
        let (proof, mut output) = circuit.prove(&input);
        log::debug!("Done generating proof");

        circuit.verify(&proof, &input, &output);

        for _ in 0..UPDATE_HEADERS_COUNT {
            let next_header = output.evm_read::<BlockHashVariable>();
            println!("next_header {:?}", next_header);
        }
    
        let total_work = output.evm_read::<WorkVariable>();
        println!("total_work {:?}", total_work);
    }

    #[test]
    fn test_verify() {
        const UPDATE_HEADERS_COUNT: usize = 10;
        let height = 200000;
        let header = bytes32!("bf0e2e13fce62f3a5f15903a177ad6a258a01f164aefed7d4a03000000000000");
        test_verify_template::<UPDATE_HEADERS_COUNT>(height, header);
    }
}