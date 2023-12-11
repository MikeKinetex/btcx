use std::ops::Index;

use plonky2x::prelude::{
    CircuitBuilder, CircuitVariable,
    PlonkParameters, U256Variable, U32Variable, 
    Div, Sub, LessThanOrEqual
};

use crate::vars::*;
use crate::consts::*;

pub trait BitcoinHeaderVerify<L: PlonkParameters<D>, const D: usize> {
    fn calculate_hash(
        &mut self,
        header_bytes: &HeaderBytesVariable
    ) -> BlockHashVariable;

    fn validate_mantissa(
        &mut self,
        threshold_int_bytes: &ThresholdIntBytesVariable,
        difficulty_exp: &U32Variable,
    ) -> ThresholdVariable;
    
    fn get_threshold(
        &mut self,
        header_bytes: &HeaderBytesVariable
    ) -> ThresholdVariable;

    fn validate_threshold(
        &mut self,
        threshold_bytes: &ThresholdVariable,
        block_hash: BlockHashVariable
    ) -> U256Variable;

    fn calculate_work(
        &mut self,
        threshold: &U256Variable,
    ) -> U256Variable;

    fn validate_header(
        &mut self,
        header: &BitcoinHeaderVariable
    );
}


impl<L: PlonkParameters<D>, const D: usize> BitcoinHeaderVerify<L, D> for CircuitBuilder<L, D> {
    fn calculate_hash(
        &mut self,
        header_bytes: &HeaderBytesVariable
    ) -> BlockHashVariable {
        let sha256_1 = self.curta_sha256(&header_bytes.0);
        self.curta_sha256(&sha256_1.as_bytes())
    }

    // Validate mantissa and exponent bits of difficulty
    fn validate_mantissa(
        &mut self,
        threshold_int_bytes: &ThresholdIntBytesVariable,
        difficulty_exp: &U32Variable
    ) -> ThresholdVariable {
        let _true = self._true();
        let const_32 = self.constant::<U32Variable>(32);
        let const_1 = self.one::<U32Variable>();

        let threshold_bytes = self.init::<ThresholdVariable>();

        // Check each threshold byte is maps to a mantissa byte or is zero
        // This is because above we only assigned threshold bytes to be the
        // 72-74th bits of the header
        for j in 0..32 {
            let const_index = self.constant::<U32Variable>(j as u32);
            let byte = threshold_int_bytes[j];

            let is_zero = self.is_zero(byte.variable);

            let index1 = self.sub(const_32, *difficulty_exp);
            let is_first_mantissa_byte = self.is_equal(const_index, index1);

            let index2 = self.add(const_1, index1);
            let is_second_mantissa_byte = self.is_equal(const_index, index2);

            let index3 = self.add(const_1, index2);
            let is_third_mantissa_byte = self.is_equal(const_index, index3);

            let range_check1 = self.or(is_first_mantissa_byte, is_second_mantissa_byte);
            let range_check2 = self.or(range_check1, is_third_mantissa_byte);
            let in_range_or_zero = self.or(range_check2, is_zero);

            self.assert_is_equal(in_range_or_zero, _true);

            let threshold_byte = U32Variable::from_be_bits(
                &threshold_bytes.as_bytes().index(j).as_be_bits(),
                self
            );
            self.connect(byte, threshold_byte);
        }

        threshold_bytes
    }

    fn get_threshold(
        &mut self,
        header_bytes: &HeaderBytesVariable
    ) -> ThresholdVariable {
        // Extract difficulty exponent from header
        let difficulty_exp = U32Variable::from_be_bits(
            &header_bytes.index(EXP_BYTE_INDEX).as_be_bits(), 
            self
        );

        let threshold_int_bytes = self.init::<ThresholdIntBytesVariable>();

        let mut assign_threshold_byte = |threshold_byte_index: u32, header_byte_index: usize| {
            let threshold_byte_idx = self.constant::<U32Variable>(threshold_byte_index);
            let access_idx = threshold_byte_idx.sub(difficulty_exp, self);
            
            let threshold_byte = self.select_array_random_gate::<U32Variable>(&threshold_int_bytes.as_slice(), access_idx.variable);
            let header_byte = U32Variable::from_be_bits(&header_bytes.index(header_byte_index).as_be_bits(), self);
            
            self.connect(threshold_byte, header_byte);
        };

        assign_threshold_byte(32, MANTISSA_FIRST_BYTE_INDEX);
        assign_threshold_byte(33, MANTISSA_SECOND_BYTE_INDEX);
        assign_threshold_byte(34, MANTISSA_THIRD_BYTE_INDEX);

        self.validate_mantissa(&threshold_int_bytes, &difficulty_exp)
    }

    fn validate_threshold(
        &mut self,
        threshold_bytes: &ThresholdVariable,
        block_hash: BlockHashVariable
    ) -> U256Variable {
        // convert threshold to u256
        let threshold = threshold_bytes.as_u256(self);

        // reverse hash bytes
        let mut hash_rev = block_hash.as_bytes();
        hash_rev.reverse();
        
        // convert hash to u256
        let hash_u = BlockHashVariable::from(hash_rev).as_u256(self);
    
        // compare hash with threshold
        let is_less = hash_u.lte(threshold, self);
        let _true = self._true();
        self.assert_is_equal(is_less, _true);

        threshold
    }

    // Calculate work given threshold bits
    // Bitcoin's formula for work W is defined as
    // W := 2**256 // threshold
    fn calculate_work(
        &mut self,
        threshold: &U256Variable,
    ) -> WorkVariable {
        let limbs = (0..8)
            .map(|_| {
                U32Variable::constant(self, u32::MAX).variable
            })
            .collect::<Vec<_>>();
        let numerator = U256Variable::from_variables_unsafe(&limbs.as_slice());        
        numerator.div(*threshold, self)
    }

    fn validate_header(
        &mut self,
        header: &BitcoinHeaderVariable
    ) {
        // calculate hash
        let hash = self.calculate_hash(&header.raw);

        // validate threshold
        let threshold = self.get_threshold(&header.raw);
        let threshold_u = self.validate_threshold(&threshold, header.hash);

        // calculate work
        let work = self.calculate_work(&threshold_u);

        self.assert_is_equal(hash, header.hash);
        self.assert_is_equal(threshold, header.threshold);
        self.assert_is_equal(work, header.work);
    }
}


#[cfg(test)]
mod test {
    use std::env;
    use ethers::types::{U256, H256};

    use plonky2x::prelude::{bytes, bytes32, DefaultBuilder, U256Variable};

    use crate::header::BitcoinHeaderVerify;
    use crate::utils::*;
    use crate::vars::*;

    #[test]
    fn single_header_verification() {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        log::debug!("Defining circuit");
        let mut builder = DefaultBuilder::new();

        let hash = builder.read::<BlockHashVariable>();
        let threshold = builder.read::<ThresholdVariable>();
        let work = builder.read::<U256Variable>();
        let header_bytes = builder.read::<HeaderBytesVariable>();

        let header = BitcoinHeaderVariable {
            hash,
            threshold,
            work,
            raw: header_bytes
        };

        builder.validate_header(&header);
        
        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut input = circuit.input();
        
        let expected_hash = bytes32!("71656d622506312551c033e123d00c4f2bc3742523ba00000000000000000000");
        let header_input = bytes!("00c05031fe0c7ab1158d234b8109d23004770f907ce86dd2602600000000000000000000d4a3d278e4427cd05d83889eee4a74e0d8e88d29b580cd4af081eeca3e5e9be1f85570655024041726249cdd");

        let (exp, mantissa) = compute_exp_and_mantissa(header_input);
        let threshold_input = compute_threshold(exp, mantissa);
        let work_input = compute_work(threshold_input);
        
        log::debug!("Exponent: {:?}", exp);
        log::debug!("Mantissa: {:?}", mantissa);
        log::debug!("Work: {:?}", work_input);
        
        input.write::<BlockHashVariable>(expected_hash);
        input.write::<ThresholdVariable>(H256::from_slice(&bits_to_bytes32(threshold_input)));
        input.write::<WorkVariable>(U256::from_little_endian(work_input.to_bytes_le().as_slice()));
        input.write::<HeaderBytesVariable>(header_input);

        log::debug!("Generating circuit proof");
        let (proof, output) = circuit.prove(&input);
        log::debug!("Done generating circuit proof");

        log::debug!("Input:{:?} ", input);
        log::debug!("Output: {:?}", output);

        log::debug!("Verifying circuit proof");
        circuit.verify(&proof, &input, &output);
        log::debug!("Done verifying circuit proof");
    }
}