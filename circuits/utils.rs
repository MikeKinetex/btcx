use std::ops::AddAssign;
use std::ops::MulAssign;
use num_bigint::BigUint;

use crate::consts::*;

pub fn compute_exp_and_mantissa(header_bytes: [u8; HEADER_BYTES_LENGTH]) -> (u32, u64) {
    let exp = (header_bytes[HEADER_EXP_BYTE_INDEX] as u32) - 3;

    let mut mantissa = 0;
    mantissa += header_bytes[HEADER_MANTISSA_THIRD_BYTE_INDEX] as u64;
    mantissa += (header_bytes[HEADER_MANTISSA_SECOND_BYTE_INDEX] as u64) << 8;
    mantissa += (header_bytes[HEADER_MANTISSA_FIRST_BYTE_INDEX] as u64) << 16;

    (exp, mantissa)
}

pub fn compute_threshold(exp: u32, mantissa: u64) -> BigUint {
    let mut threshold_bits = [false; 256];

    for i in 0..256 {
        if (i as u32) < 256 - (exp * 8) && mantissa as u128 & (1u128 << (255u128 - (exp * 8) as u128 - i as u128)) != 0 {
            threshold_bits[i] = true;
        }
    }

    BigUint::from_bytes_be(
        &bits_to_bytes32(threshold_bits)
    )
}

pub fn compute_work(threshold: BigUint) -> BigUint {
    let mut acc = BigUint::new(vec![1]);
    let mut denominator = acc.clone();
    denominator.add_assign(threshold);

    for _ in 0..256 {
        acc.mul_assign(BigUint::new(vec![2]));
    }
    let numerator = acc;
    
    numerator / denominator
}

pub fn adjust_threshold(threshold: BigUint, period_start_time: u32, period_end_time: u32) -> BigUint {
    let pow_target_timespan = 14 * 24 * 60 * 60;
    let pow_limit = BigUint::from_bytes_be(&[0xff; 28]);
            
    let timespan = period_end_time - period_start_time;
    let timespan = if timespan < pow_target_timespan / 4 {
        pow_target_timespan / 4
    } else if timespan > pow_target_timespan * 4 {
        pow_target_timespan * 4
    } else {
        timespan
    };

    let mut new_threshold = threshold * timespan / pow_target_timespan;
    if new_threshold > pow_limit {
        new_threshold = pow_limit;
    }

    new_threshold
}

pub fn bits_to_bytes32(bits: [bool; 256]) -> [u8; 32] {
    let mut bytes = [0; 32];
    for i in 0..256 {
        bytes[i / 8] |= u8::from(bits[i]) << (7 - i % 8);
    }
    bytes
}