use std::ops::Index;

use plonky2x::prelude::{
    BoolVariable, ByteVariable, Bytes32Variable, BytesVariable, CircuitBuilder, LessThanOrEqual,
    PlonkParameters, U32Variable,
};

use crate::consts::*;
use crate::vars::*;

pub trait BitcoinHeaderVerify<L: PlonkParameters<D>, const D: usize> {
    fn calculate_hash(&mut self, header_bytes: &HeaderBytesVariable) -> BlockHashVariable;

    fn get_parent_hash(&mut self, header: &HeaderBytesVariable) -> BlockHashVariable;

    fn get_merkle_root(&mut self, header: &HeaderBytesVariable) -> Bytes32Variable;

    fn get_timestamp(&mut self, header: &HeaderBytesVariable) -> U32Variable;

    fn get_threshold(&mut self, header_bytes: &HeaderBytesVariable) -> ThresholdVariable;

    fn validate_threshold(
        &mut self,
        threshold: &ThresholdVariable,
        block_hash: BlockHashVariable,
    ) -> BoolVariable;

    fn validate_header(&mut self, header_bytes: &HeaderBytesVariable) -> BitcoinHeaderVariable;
}

impl<L: PlonkParameters<D>, const D: usize> BitcoinHeaderVerify<L, D> for CircuitBuilder<L, D> {
    fn calculate_hash(&mut self, header_bytes: &HeaderBytesVariable) -> BlockHashVariable {
        let sha256_1 = self.curta_sha256(&header_bytes.0);
        self.curta_sha256(&sha256_1.as_bytes())
    }

    fn get_parent_hash(&mut self, header_bytes: &HeaderBytesVariable) -> BlockHashVariable {
        header_bytes[HEADER_PARENT_HASH_INDEX..HEADER_PARENT_HASH_INDEX + 32]
            .try_into()
            .unwrap()
    }

    fn get_merkle_root(&mut self, header_bytes: &HeaderBytesVariable) -> Bytes32Variable {
        header_bytes[HEADER_MERKLE_ROOT_INDEX..HEADER_MERKLE_ROOT_INDEX + 32]
            .try_into()
            .unwrap()
    }

    fn get_timestamp(&mut self, header_bytes: &HeaderBytesVariable) -> U32Variable {
        U32Variable::from_be_bits(
            &header_bytes[HEADER_TIMESTAMP_INDEX..HEADER_TIMESTAMP_INDEX + 4]
                .iter()
                .rev()
                .flat_map(|byte| byte.as_be_bits())
                .collect::<Vec<_>>(),
            self,
        )
    }

    fn get_threshold(&mut self, header_bytes: &HeaderBytesVariable) -> ThresholdVariable {
        // Extract difficulty exponent from header
        let difficulty_exp = U32Variable::from_be_bits(
            &header_bytes.index(HEADER_EXP_BYTE_INDEX).as_be_bits(),
            self,
        );

        let const_32 = self.constant::<U32Variable>(32);
        let const_1 = self.one::<U32Variable>();

        let mantissa_index_1 = self.sub(const_32, difficulty_exp);
        let mantissa_index_2 = self.add(const_1, mantissa_index_1);
        let mantissa_index_3 = self.add(const_1, mantissa_index_2);

        let mut threshold_bytes = Vec::<ByteVariable>::new();

        for j in 0..32 {
            let const_index = self.constant::<U32Variable>(j as u32);

            let is_first_mantissa_byte = self.is_equal(const_index, mantissa_index_1);
            let is_second_mantissa_byte = self.is_equal(const_index, mantissa_index_2);
            let is_third_mantissa_byte = self.is_equal(const_index, mantissa_index_3);

            let mut threshold_byte = self.zero::<ByteVariable>();

            threshold_byte = self.select(
                is_first_mantissa_byte,
                header_bytes[HEADER_MANTISSA_FIRST_BYTE_INDEX],
                threshold_byte,
            );
            threshold_byte = self.select(
                is_second_mantissa_byte,
                header_bytes[HEADER_MANTISSA_SECOND_BYTE_INDEX],
                threshold_byte,
            );
            threshold_byte = self.select(
                is_third_mantissa_byte,
                header_bytes[HEADER_MANTISSA_THIRD_BYTE_INDEX],
                threshold_byte,
            );

            threshold_bytes.push(threshold_byte);
        }

        Bytes32Variable(BytesVariable(
            threshold_bytes.as_slice().try_into().unwrap(),
        ))
        .as_u256(self)
    }

    fn validate_threshold(
        &mut self,
        threshold: &ThresholdVariable,
        block_hash: BlockHashVariable,
    ) -> BoolVariable {
        // reverse hash bytes
        let mut hash_rev = block_hash.as_bytes().clone();
        hash_rev.reverse();

        // convert hash to u256
        let hash_u = BlockHashVariable::from(hash_rev).as_u256(self);

        // compare hash with threshold
        hash_u.lte(*threshold, self)
    }

    fn validate_header(&mut self, header_bytes: &HeaderBytesVariable) -> BitcoinHeaderVariable {
        let _true = self._true();
        // calculate hash
        let hash = self.calculate_hash(&header_bytes);

        // get threshold
        let threshold = self.get_threshold(&header_bytes);

        // validate threshold
        let is_valid = self.validate_threshold(&threshold, hash);
        self.assert_is_equal(is_valid, _true);

        // parent hash
        let parent_hash = self.get_parent_hash(&header_bytes);

        // merkle root
        let merkle_root = self.get_merkle_root(&header_bytes);

        // timestamp
        let timestamp = self.get_timestamp(&header_bytes);

        // return hash & work
        BitcoinHeaderVariable {
            hash,
            parent_hash,
            merkle_root,
            timestamp,
            threshold,
        }
    }
}

#[cfg(test)]
mod test {
    use ethers::types::U256;
    use std::env;

    use plonky2x::prelude::{bytes, bytes32, DefaultBuilder};

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_validate_header() {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        log::debug!("Defining circuit");
        let mut builder = DefaultBuilder::new();

        let header_bytes = builder.read::<HeaderBytesVariable>();
        let header = builder.validate_header(&header_bytes);
        builder.write(header);

        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let header_input = bytes!("00c05031fe0c7ab1158d234b8109d23004770f907ce86dd2602600000000000000000000d4a3d278e4427cd05d83889eee4a74e0d8e88d29b580cd4af081eeca3e5e9be1f85570655024041726249cdd");

        let mut input = circuit.input();
        input.write::<HeaderBytesVariable>(header_input);

        log::debug!("Generating circuit proof");
        let (proof, output) = circuit.prove(&input);
        log::debug!("Done generating circuit proof");

        log::debug!("Verifying circuit proof");
        circuit.verify(&proof, &input, &output);
        log::debug!("Done verifying circuit proof");

        let mut _output = output.clone();
        let header = _output.read::<BitcoinHeaderVariable>();

        let expected_hash =
            bytes32!("71656d622506312551c033e123d00c4f2bc3742523ba00000000000000000000");
        let expected_parent_hash =
            bytes32!("0xfe0c7ab1158d234b8109d23004770f907ce86dd2602600000000000000000000");
        let expected_merkle_root =
            bytes32!("0xd4a3d278e4427cd05d83889eee4a74e0d8e88d29b580cd4af081eeca3e5e9be1");
        let expected_timestamp = 1701860856;
        let (exp, mantissa) = compute_exp_and_mantissa(header_input);
        let expected_threshold =
            U256::from_little_endian(compute_threshold(exp, mantissa).to_bytes_le().as_slice());

        log::debug!("Hash: {:?} = {:?}", header.hash, expected_hash);
        log::debug!(
            "Parent hash: {:?} = {:?}",
            header.parent_hash,
            expected_parent_hash
        );
        log::debug!(
            "Merke root: {:?} = {:?}",
            header.merkle_root,
            expected_merkle_root
        );
        log::debug!(
            "Timestamp: {:?} = {:?}",
            header.timestamp,
            expected_timestamp
        );
        log::debug!(
            "Threshold: {:?} = {:?}",
            header.threshold,
            expected_threshold
        );

        assert_eq!(header.hash, expected_hash);
        assert_eq!(header.parent_hash, expected_parent_hash);
        assert_eq!(header.merkle_root, expected_merkle_root);
        assert_eq!(header.timestamp, expected_timestamp);
        assert_eq!(header.threshold, expected_threshold);
    }
}
