use plonky2x::backend::circuit::Circuit;
use plonky2x::frontend::hint::simple::hint::Hint;
use plonky2x::prelude::{
    bytes,
    ArrayVariable, U64Variable,
    CircuitBuilder, PlonkParameters,
    ValueStream, VariableStream
};

use serde::{Deserialize, Serialize};

use crate::multi::BitcoinMultiVerify;
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

        let update_headers_bytes = vec![
            bytes!("0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c"),
            bytes!("010000006fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000982051fd1e4ba744bbbe680e1fee14677ba1a3c3540bf7b1cdb606e857233e0e61bc6649ffff001d01e36299"),
            bytes!("010000004860eb18bf1b1620e37e9490fc8a427514416fd75159ab86688e9a8300000000d5fdcc541e25de1c7a5addedf24858b8bb665c9f36ef744ee42c316022c90f9bb0bc6649ffff001d08d2bd61"),
            bytes!("01000000bddd99ccfda39da1b108ce1a5d70038d0a967bacb68b6b63065f626a0000000044f672226090d85db9a9f2fbfe5f0f9609b387af7be5b7fbb7a1767c831c9e995dbe6649ffff001d05e0ed6d"),
            bytes!("010000004944469562ae1c2c74d9a535e00b6f3e40ffbad4f2fda3895501b582000000007a06ea98cd40ba2e3288262b28638cec5337c1456aaf5eedc8e9e5a20f062bdf8cc16649ffff001d2bfee0a9"),
            bytes!("0100000085144a84488ea88d221c8bd6c059da090e88f8a2c99690ee55dbba4e00000000e11c48fecdd9e72510ca84f023370c9a38bf91ac5cae88019bee94d24528526344c36649ffff001d1d03e477"),
            bytes!("01000000fc33f596f822a0a1951ffdbf2a897b095636ad871707bf5d3162729b00000000379dfb96a5ea8c81700ea4ac6b97ae9a9312b2d4301a29580e924ee6761a2520adc46649ffff001d189c4c97"),
            bytes!("010000008d778fdc15a2d3fb76b7122a3b5582bea4f21f5a0c693537e7a03130000000003f674005103b42f984169c7d008370967e91920a6a5d64fd51282f75bc73a68af1c66649ffff001d39a59c86"),
            bytes!("010000004494c8cf4154bdcc0720cd4a59d9c9b285e4b146d45f061d2b6c967100000000e3855ed886605b6d4a99d5fa2ef2e9b0b164e63df3c4136bebf2d0dac0f1f7a667c86649ffff001d1c4b5666"),
            bytes!("01000000c60ddef1b7618ca2348a46e868afc26e3efc68226c78aa47f8488c4000000000c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd37047fca6649ffff001d28404f53"),
        ];

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
        let header = bytes32!("0000000000000000000000000000000000000000000000000000000000000000");
        let height = 0;
        test_verify_template::<UPDATE_HEADERS_COUNT>(height, header);
    }
}