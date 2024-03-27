use plonky2x::prelude::{
    CircuitBuilder, PlonkParameters, 
    CircuitVariable, ArrayVariable,
    U256Variable, U32Variable, U64Variable
};

use crate::builder::header::BitcoinHeaderVerify;
use crate::vars::*;

pub trait BitcoinMultiVerify<L: PlonkParameters<D>, const D: usize> {
    fn validate_headers<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: &U64Variable,
        prev_header_hash: &BlockHashVariable,
        threshold: &ThresholdVariable,
        update_headers_bytes: &ArrayVariable<
            HeaderBytesVariable,
            UPDATE_HEADERS_COUNT
        >
    ) -> ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>;

    fn validate_headers_with_retargeting<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: &U64Variable,
        prev_header_hash: &BlockHashVariable,
        period_start_hash: &BlockHashVariable,
        period_start_header_bytes: &HeaderBytesVariable,
        period_end_header_bytes: &HeaderBytesVariable,
        threshold: &ThresholdVariable,
        update_headers_bytes: &ArrayVariable<
            HeaderBytesVariable,
            UPDATE_HEADERS_COUNT
        >
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, ThresholdVariable);

    fn adjust_threshold(
        &mut self,
        threshold: &ThresholdVariable,
        period_start_timestamp: U32Variable,
        period_end_timestamp: U32Variable
    ) -> U256Variable;
}

impl<L: PlonkParameters<D>, const D: usize> BitcoinMultiVerify<L, D> for CircuitBuilder<L, D> {
    fn validate_headers<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: &U64Variable,
        prev_header_hash: &BlockHashVariable,
        threshold: &ThresholdVariable,
        update_headers_bytes: &ArrayVariable<
            HeaderBytesVariable,
            UPDATE_HEADERS_COUNT
        >
    ) -> ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT> {
        let _true = self._true();

        // check if provided blocks are in bounds
        let retarget_window = self.constant::<U64Variable>(2016);

        let _m = self.rem(*prev_block_number, retarget_window);        
        let blocks_to_retarget = self.sub(retarget_window, _m);
        let last_block_in_period = self.add(*prev_block_number, blocks_to_retarget);

        let block_count = self.constant::<U64Variable>(UPDATE_HEADERS_COUNT as u64);
        let last_block_number = self.add(*prev_block_number, block_count);

        let is_last_block_in_bounds = self.lt(last_block_number, last_block_in_period);
        self.assert_is_equal(is_last_block_in_bounds, _true);

        // validate headers
        let mut hashes: Vec<BlockHashVariable> = Vec::new();

        for h in 0..UPDATE_HEADERS_COUNT {
            let header = self.validate_header(&update_headers_bytes[h]);
            
            self.assert_is_equal(*threshold, header.threshold);
            self.assert_is_equal(
                if h == 0 { *prev_header_hash } else { hashes[h - 1] }, 
                header.parent_hash
            );

            hashes.push(header.hash);
        }

        ArrayVariable::from(hashes)
    }

    fn validate_headers_with_retargeting<const UPDATE_HEADERS_COUNT: usize>(
            &mut self,
            prev_block_number: &U64Variable,
            prev_header_hash: &BlockHashVariable,
            period_start_hash: &BlockHashVariable,
            period_start_header_bytes: &HeaderBytesVariable,
            // period_end_hash: &BlockHashVariable,
            period_end_header_bytes: &HeaderBytesVariable,
            threshold: &ThresholdVariable,
            update_headers_bytes: &ArrayVariable<
                HeaderBytesVariable,
                UPDATE_HEADERS_COUNT
            >
        ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, ThresholdVariable) {
        let _true = self._true();
        let _zero = self.zero::<U64Variable>();
        
        // check if provided blocks are in bounds
        let retarget_window = self.constant::<U64Variable>(2016);

        let _m = self.rem(*prev_block_number, retarget_window);        
        let blocks_to_retarget = self.sub(retarget_window, _m);
        let last_block_in_period = self.add(*prev_block_number, blocks_to_retarget);
        let max_block_number = self.add(last_block_in_period, retarget_window);

        let block_count = self.constant::<U64Variable>(UPDATE_HEADERS_COUNT as u64);
        let last_block_number = self.add(*prev_block_number, block_count);

        let is_last_block_in_hbounds  = self.lte(last_block_number, max_block_number);
        self.assert_is_equal(is_last_block_in_hbounds, _true);

        let is_last_block_in_lbounds = self.gt(last_block_number, last_block_in_period);
        self.assert_is_equal(is_last_block_in_lbounds, _true);

        // validate period start header
        let period_start_header = self.validate_header(&period_start_header_bytes);
        self.assert_is_equal(*period_start_hash, period_start_header.hash);
        self.assert_is_equal(*threshold, period_start_header.threshold);

        // get next period difficulty
        let period_end_header = self.validate_header(&period_end_header_bytes);
        let next_threshold: U256Variable = self.adjust_threshold(
            threshold,
            period_start_header.timestamp,
            period_end_header.timestamp
        );

        // validate headers
        let mut hashes: Vec<BlockHashVariable> = Vec::new();
        
        for h in 0..UPDATE_HEADERS_COUNT {
            let _h = self.constant::<U64Variable>(h as u64);
            let block_number = self.add(*prev_block_number, _h);
            let is_in_prev_period = self.lte(block_number, last_block_in_period);
            
            let current_threshold = self.select(is_in_prev_period, *threshold, next_threshold);

            let header = self.validate_header(&update_headers_bytes[h]);
            
            self.assert_is_equal(current_threshold, header.threshold);
            self.assert_is_equal(
                if h == 0 { *prev_header_hash } else { hashes[h - 1] }, 
                header.parent_hash
            );

            hashes.push(header.hash);
        }

        (ArrayVariable::from(hashes), next_threshold)        
    }

    fn adjust_threshold(
            &mut self,
            threshold: &ThresholdVariable,
            period_start_timestamp: U32Variable,
            period_end_timestamp: U32Variable
        ) -> U256Variable {
        let pow_ts_min = self.constant::<U32Variable>(2016 * 600 / 4);
        let pow_ts_max = self.constant::<U32Variable>(2016 * 600 * 4);

        let pwt_limbs = (0..8)
            .map(|i| {
                self.constant::<U32Variable>(if i == 0 { 2016 * 600 } else { 0 }).variable
            })
            .collect::<Vec<_>>();
        let pow_ts = U256Variable::from_variables_unsafe(&pwt_limbs.as_slice()); 

        let pwl_limbs = (0..8)
            .map(|i| {
                self.constant::<U32Variable>(if i == 0 { 0 } else { u32::MAX }).variable
            })
            .collect::<Vec<_>>();
        let pow_limit = U256Variable::from_variables_unsafe(&pwl_limbs.as_slice());     

        let timespan = self.sub(period_end_timestamp, period_start_timestamp);

        let is_pow_ts_min = self.lt(timespan, pow_ts_min);
        let is_pow_ts_max = self.gt(timespan, pow_ts_max);

        let ts = self.select(is_pow_ts_min, pow_ts_min, timespan);
        let ts_u32 = self.select(is_pow_ts_max, pow_ts_max, ts);

        let ts_limbs = (0..8)
            .map(|i| {
                if i == 0 { ts_u32.variable } else { self.zero::<U32Variable>().variable }
            })
            .collect::<Vec<_>>();
        let timespan_adjusted = U256Variable::from_variables_unsafe(&ts_limbs.as_slice());      

        let dividend = self.mul(*threshold, timespan_adjusted);

        let new_target = self.div(dividend,pow_ts);
        let is_lower_pow_limit = self.is_zero(new_target.limbs[7].variable);

        self.select(is_lower_pow_limit, new_target, pow_limit)
    }
}


#[cfg(test)]
mod test {
    use std::env;
    use ethers::types::{H256, U256};

    use plonky2x::prelude::{
        bytes, bytes32,
        DefaultBuilder
    };

    use crate::consts::*;
    use super::*;

    #[test]
    fn test_validate_headers() {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        log::debug!("Defining circuit");
        let mut builder = DefaultBuilder::new();
        
        let prev_block_number = builder.read::<U64Variable>();
        let prev_header_hash = builder.read::<BlockHashVariable>();
        let threshold = builder.read::<ThresholdVariable>();
        let update_headers_bytes = builder.read::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>();
        
        let update_headers_hashes = builder.validate_headers(
            &prev_block_number,
            &prev_header_hash,
            &threshold,
            &update_headers_bytes
        );
        
        builder.write(update_headers_hashes);

        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut input = circuit.input();
        input.write::<U64Variable>(0);
        input.write::<BlockHashVariable>(mock_prev_header_hash());
        input.write::<ThresholdVariable>(mock_threshold());
        input.write::<ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>>(mock_headers_bytes());
        
        log::debug!("Generating circuit proof");
        let (proof, mut output) = circuit.prove(&input);
        log::debug!("Done verifying circuit proof");

        log::debug!("Verifying circuit proof");
        circuit.verify(&proof, &input, &output);
        log::debug!("Done verifying circuit proof");

        let headers_hashes = output.read::<ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>>();
        let expected_hashes = mock_expected_headers_hashes();

        assert!(headers_hashes[0] == expected_hashes[0]);
        assert!(headers_hashes[UPDATE_HEADERS_COUNT - 1] == expected_hashes[UPDATE_HEADERS_COUNT - 1]);
    }

    fn mock_prev_header_hash() -> H256 {
        bytes32!("0000000000000000000000000000000000000000000000000000000000000000")
    }

    fn mock_threshold() -> U256 {
        U256::from_dec_str("26959535291011309493156476344723991336010898738574164086137773096960").unwrap()
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
}