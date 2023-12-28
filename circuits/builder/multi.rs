use plonky2x::prelude::{
    ArrayVariable, CircuitBuilder, PlonkParameters
};

use crate::builder::header::BitcoinHeaderVerify;
use crate::consts::*;
use crate::vars::*;

pub trait BitcoinMultiVerify<L: PlonkParameters<D>, const D: usize> {
    fn get_parent_hash(
        &mut self,
        header: &HeaderBytesVariable
    ) -> BlockHashVariable;

    fn validate_headers<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_header_hash: &BlockHashVariable,
        update_headers_bytes: &ArrayVariable<
            HeaderBytesVariable,
            UPDATE_HEADERS_COUNT
        >
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, WorkVariable);
}

impl<L: PlonkParameters<D>, const D: usize> BitcoinMultiVerify<L, D> for CircuitBuilder<L, D> {
    fn get_parent_hash(
        &mut self,
        header: &HeaderBytesVariable
    ) -> BlockHashVariable {
        header[HEADER_PARENT_HASH_INDEX..HEADER_PARENT_HASH_INDEX + 32].try_into().unwrap()
    }

    fn validate_headers<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_header_hash: &BlockHashVariable,
        update_headers_bytes: &ArrayVariable<
            HeaderBytesVariable,
            UPDATE_HEADERS_COUNT
        >
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, WorkVariable) {
        if UPDATE_HEADERS_COUNT < 2 {
            panic!("Not enough headers to form a chain");
        }

        let mut hashes: Vec<BlockHashVariable> = Vec::new();
        let mut work: Vec<WorkVariable> = Vec::new();

        for h in 0..UPDATE_HEADERS_COUNT {
            let header = self.validate_header(&update_headers_bytes[h]);
            hashes.push(header.hash);

            let parent_hash = self.get_parent_hash(&update_headers_bytes[h]);

            if h == 0 {
                self.assert_is_equal(*prev_header_hash, parent_hash);
                work.push(header.work);
            } else {
                self.assert_is_equal(hashes[h - 1], parent_hash);
                work.push(
                    self.add(work[h - 1], header.work)
                );
            }
        }

        let total_work = work[work.len() - 2];

        (ArrayVariable::from(hashes), total_work)
    }
}


#[cfg(test)]
mod test {
    use std::env;
    use ethers::types::H256;

    use plonky2x::prelude::{
        bytes, bytes32,
        DefaultBuilder, ArrayVariable
    };

    use crate::consts::*;
    use super::*;

    #[test]
    fn test_validate_headers() {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        log::debug!("Defining circuit");
        let mut builder = DefaultBuilder::new();
        
        let prev_header_hash = builder.read::<BlockHashVariable>();
        let update_headers_bytes = builder.read::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>();
        let (update_headers_hashes, update_total_work) = builder.validate_headers(&prev_header_hash, &update_headers_bytes);
        
        builder.write(update_headers_hashes);
        builder.write(update_total_work);

        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut input = circuit.input();
        input.write::<BlockHashVariable>(mock_prev_header_hash());
        input.write::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(mock_headers_bytes());
        
        log::debug!("Generating circuit proof");
        let (proof, mut output) = circuit.prove(&input);
        log::debug!("Done verifying circuit proof");

        log::debug!("Verifying circuit proof");
        circuit.verify(&proof, &input, &output);
        log::debug!("Done verifying circuit proof");

        let headers_hashes = output.read::<ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>>();
        let total_work = output.read::<WorkVariable>();

        let expected_hashes = mock_expected_headers_hashes();

        assert!(headers_hashes[0] == expected_hashes[0]);
        assert!(headers_hashes[UPDATE_HEADERS_COUNT - 1] == expected_hashes[UPDATE_HEADERS_COUNT - 1]);
        assert!(total_work.as_u64() == mock_expected_total_work());
    }

    fn mock_prev_header_hash() -> H256 {
        bytes32!("0000000000000000000000000000000000000000000000000000000000000000")
    }

    fn mock_headers_bytes() -> Vec<[u8; HEADER_BYTES_LENGTH]> {
        vec![
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
        ]
    }

    fn mock_expected_headers_hashes() -> Vec<H256> {
        vec![
            bytes32!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"),
            bytes32!("4860eb18bf1b1620e37e9490fc8a427514416fd75159ab86688e9a8300000000"),
            bytes32!("bddd99ccfda39da1b108ce1a5d70038d0a967bacb68b6b63065f626a00000000"),
            bytes32!("4944469562ae1c2c74d9a535e00b6f3e40ffbad4f2fda3895501b58200000000"),
            bytes32!("85144a84488ea88d221c8bd6c059da090e88f8a2c99690ee55dbba4e00000000"),
            bytes32!("fc33f596f822a0a1951ffdbf2a897b095636ad871707bf5d3162729b00000000"),
            bytes32!("8d778fdc15a2d3fb76b7122a3b5582bea4f21f5a0c693537e7a0313000000000"),
            bytes32!("4494c8cf4154bdcc0720cd4a59d9c9b285e4b146d45f061d2b6c967100000000"),
            bytes32!("c60ddef1b7618ca2348a46e868afc26e3efc68226c78aa47f8488c4000000000"),
            bytes32!("0508085c47cc849eb80ea905cc7800a3be674ffc57263cf210c59d8d00000000"),
        ]
    }
    
    fn mock_expected_total_work() -> u64 {
        38655295497
    }
}