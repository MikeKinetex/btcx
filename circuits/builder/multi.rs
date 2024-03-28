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
                self.constant::<U32Variable>(if i == 7 { 0 } else { u32::MAX }).variable
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
    use std::str::FromStr;
    use ethers::types::U256;

    use num_bigint::BigUint;
    use plonky2x::prelude::DefaultBuilder;

    use crate::utils::*;
    use super::*;

    fn test_adjust_threshold_template(
        period_threshold: &str,
        period_start_ts: u32,
        period_end_ts: u32
    ) {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        log::debug!("Defining circuit");
        let mut builder = DefaultBuilder::new();

        let threshold = builder.read::<ThresholdVariable>();
        let period_start_timestamp = builder.read::<U32Variable>();
        let period_end_timestamp = builder.read::<U32Variable>();

        let adjusted_threshold = builder.adjust_threshold(
            &threshold,
            period_start_timestamp,
            period_end_timestamp
        );
        builder.write(adjusted_threshold);
    
        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let period_threshold_u256 = U256::from_dec_str(period_threshold).unwrap();
        
        let mut input = circuit.input();
        input.write::<ThresholdVariable>(period_threshold_u256);
        input.write::<U32Variable>(period_start_ts);
        input.write::<U32Variable>(period_end_ts);

        log::debug!("Generating circuit proof");
        let (proof, output) = circuit.prove(&input);
        log::debug!("Done generating circuit proof");

        log::debug!("Verifying circuit proof");
        circuit.verify(&proof, &input, &output);
        log::debug!("Done verifying circuit proof");

        let mut _output = output.clone();
        let adjusted_threshold = _output.read::<ThresholdVariable>();

        let expected_threshold = U256::from_little_endian(
            adjust_threshold(
                BigUint::from_str(period_threshold).unwrap(),
                period_start_ts, 
                period_end_ts
            ).to_bytes_le().as_slice()
        );

        log::debug!("Adjusted threshold: {:?} = {:?}", adjusted_threshold, expected_threshold);
        assert_eq!(adjusted_threshold, expected_threshold);
    }

    #[test]
    fn test_adjust_threshold_201600() {
        test_adjust_threshold_template(
            "8825801199382903987726989797449454220615414953524072026210304",
            1349226660,
            1350429295
        );
    }

    #[test]
    fn test_adjust_threshold_powlimit() {
        test_adjust_threshold_template(
            "26959946667150639794667015087019630673637144422540572481103610249215",
            0,
            2419200 // 14 * 24 * 60 * 60 * 2
        );
    }
}