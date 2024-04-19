use plonky2x::prelude::{
    ArrayVariable, CircuitBuilder, PlonkParameters, U256Variable, U32Variable, U64Variable,
};

use crate::builder::header::BitcoinHeaderVerify;
use crate::utils::u256_from_gen;
use crate::vars::*;

pub trait BitcoinMultiVerify<L: PlonkParameters<D>, const D: usize> {
    fn validate_headers<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_header_hash: &BlockHashVariable,
        threshold: &ThresholdVariable,
        update_headers_bytes: &ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>,
    ) -> ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>;

    fn validate_headers_with_retargeting<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: &U64Variable,
        prev_header_hash: &BlockHashVariable,
        period_start_hash: &BlockHashVariable,
        curret_threshold: &ThresholdVariable,
        next_threshold: &ThresholdVariable,
        period_start_header_bytes: &HeaderBytesVariable,
        period_end_header_bytes: &HeaderBytesVariable,
        update_headers_bytes: &ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>,
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, ThresholdVariable);

    fn adjust_threshold(
        &mut self,
        threshold: &ThresholdVariable,
        period_start_timestamp: U32Variable,
        period_end_timestamp: U32Variable,
    ) -> U256Variable;
}

impl<L: PlonkParameters<D>, const D: usize> BitcoinMultiVerify<L, D> for CircuitBuilder<L, D> {
    fn validate_headers<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_header_hash: &BlockHashVariable,
        threshold: &ThresholdVariable,
        update_headers_bytes: &ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>,
    ) -> ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT> {
        let mut hashes: Vec<BlockHashVariable> = Vec::new();

        for h in 0..UPDATE_HEADERS_COUNT {
            let header = self.validate_header(&update_headers_bytes[h]);

            self.assert_is_equal(*threshold, header.threshold);
            self.assert_is_equal(
                if h == 0 {
                    *prev_header_hash
                } else {
                    hashes[h - 1]
                },
                header.parent_hash,
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
        current_threshold: &ThresholdVariable,
        next_threshold: &ThresholdVariable,
        period_start_header_bytes: &HeaderBytesVariable,
        period_end_header_bytes: &HeaderBytesVariable,
        update_headers_bytes: &ArrayVariable<HeaderBytesVariable, UPDATE_HEADERS_COUNT>,
    ) -> (ArrayVariable<BlockHashVariable, UPDATE_HEADERS_COUNT>, ThresholdVariable) {
        // constants
        let _zero = self.zero::<U64Variable>();
        let _one = self.one::<U64Variable>();
        let retarget_window = self.constant::<U64Variable>(2016);

        // calculate index of the first block in the next period after retargeting
        let first_bn_in_seq = self.add(*prev_block_number, _one);
        let m = self.rem(first_bn_in_seq, retarget_window);
        let d = self.sub(retarget_window, m);
        let start_period_block_index = self.rem(d, retarget_window);

        // validate period start header
        let period_start_header = self.validate_header(&period_start_header_bytes);
        self.assert_is_equal(*period_start_hash, period_start_header.hash);
        self.assert_is_equal(*current_threshold, period_start_header.threshold);

        // validate period end header
        let period_end_header = self.validate_header(&period_end_header_bytes);
        let not_in_seq = self.is_equal(start_period_block_index, _zero);
        let period_end_header_hash =
            self.select(not_in_seq, *prev_header_hash, period_end_header.hash);
        self.assert_is_equal(period_end_header_hash, period_end_header.hash);
        self.assert_is_equal(*current_threshold, period_end_header.threshold);

        // retarget threshold
        let next_threshold_adjusted: U256Variable = self.adjust_threshold(
            current_threshold,
            period_start_header.timestamp,
            period_end_header.timestamp,
        );
        // refine and validate next threshold
        let next_threshold_refined = u256_from_gen(|i| {
            let lhs = self.to_be_bits(next_threshold.limbs[i]);
            let rhs = self.to_be_bits(next_threshold_adjusted.limbs[i]);
            let lrs = (0..32)
                .map(|j| self.and(lhs[j], rhs[j]))
                .collect::<Vec<_>>();
            U32Variable::from_be_bits(lrs.as_slice(), self).variable
        });
        self.assert_is_equal(*next_threshold, next_threshold_refined);

        // validate headers
        let mut hashes: Vec<BlockHashVariable> = Vec::new();

        for i in 0..UPDATE_HEADERS_COUNT {
            let index = self.constant::<U64Variable>(i as u64);
            let is_in_prev_period = self.lt(index, start_period_block_index);

            let header = self.validate_header(&update_headers_bytes[i]);

            // validate threshold
            let threshold = self.select(is_in_prev_period, *current_threshold, *next_threshold);
            self.assert_is_equal(threshold, header.threshold);

            // validate parent hash
            self.assert_is_equal(
                if i == 0 {
                    *prev_header_hash
                } else {
                    hashes[i - 1]
                },
                header.parent_hash,
            );

            // validate period end header
            let next_index = self.add(index, _one);
            let is_last_in_prev_period = self.is_equal(next_index, start_period_block_index);
            let hash = self.select(is_last_in_prev_period, period_end_header_hash, header.hash);

            hashes.push(hash);
        }

        (ArrayVariable::from(hashes), next_threshold_refined)
    }

    fn adjust_threshold(
        &mut self,
        threshold: &ThresholdVariable,
        period_start_timestamp: U32Variable,
        period_end_timestamp: U32Variable,
    ) -> U256Variable {
        let pow_ts_min = self.constant::<U32Variable>(2016 * 600 / 4);
        let pow_ts_max = self.constant::<U32Variable>(2016 * 600 * 4);

        let pow_ts = u256_from_gen(|i| {
            self.constant::<U32Variable>(if i == 0 { 2016 * 600 } else { 0 })
                .variable
        });

        let pow_limit = u256_from_gen(|i| {
            self.constant::<U32Variable>(if i == 7 { 0 } else { u32::MAX })
                .variable
        });

        let timespan = self.sub(period_end_timestamp, period_start_timestamp);

        let is_pow_ts_min = self.lt(timespan, pow_ts_min);
        let is_pow_ts_max = self.gt(timespan, pow_ts_max);

        let ts_if_min = self.select(is_pow_ts_min, pow_ts_min, timespan);
        let ts_if_max = self.select(is_pow_ts_max, pow_ts_max, ts_if_min);

        let timespan_adjusted = u256_from_gen(|i| {
            if i == 0 {
                ts_if_max.variable
            } else {
                self.zero::<U32Variable>().variable
            }
        });

        let dividend = self.mul(*threshold, timespan_adjusted);

        let new_target = self.div(dividend, pow_ts);
        let is_lower_pow_limit = self.is_zero(new_target.limbs[7].variable);

        self.select(is_lower_pow_limit, new_target, pow_limit)
    }
}

#[cfg(test)]
mod test {
    use ethers::types::U256;
    use std::env;
    use std::str::FromStr;

    use num_bigint::BigUint;
    use plonky2x::prelude::DefaultBuilder;

    use super::*;
    use crate::utils::*;

    fn test_adjust_threshold_template(
        period_threshold: &str,
        period_start_ts: u32,
        period_end_ts: u32,
    ) {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        log::debug!("Defining circuit");
        let mut builder = DefaultBuilder::new();

        let threshold = builder.read::<ThresholdVariable>();
        let period_start_timestamp = builder.read::<U32Variable>();
        let period_end_timestamp = builder.read::<U32Variable>();

        let adjusted_threshold =
            builder.adjust_threshold(&threshold, period_start_timestamp, period_end_timestamp);
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
                period_end_ts,
            )
            .to_bytes_le()
            .as_slice(),
        );

        log::debug!(
            "Adjusted threshold: {:?} = {:?}",
            adjusted_threshold,
            expected_threshold
        );
        assert_eq!(adjusted_threshold, expected_threshold);
    }

    #[test]
    fn test_adjust_threshold_201600() {
        test_adjust_threshold_template(
            "8825801199382903987726989797449454220615414953524072026210304",
            1349226660,
            1350429295,
        );
    }

    #[test]
    fn test_adjust_threshold_powlimit() {
        test_adjust_threshold_template(
            "26959946667150639794667015087019630673637144422540572481103610249215",
            0,
            2419200, // 14 * 24 * 60 * 60 * 2
        );
    }
}
