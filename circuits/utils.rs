use std::ops::AddAssign;
use std::ops::MulAssign;
use num_bigint::BigUint;

use crate::consts::*;

pub fn compute_exp_and_mantissa(header_bytes: [u8; HEADER_BYTES_LENGTH]) -> (u32, u64) {
    let exp = (header_bytes[EXP_BYTE_INDEX] as u32) - 3;

    let mut mantissa = 0;
    mantissa += header_bytes[MANTISSA_THIRD_BYTE_INDEX] as u64;
    mantissa += (header_bytes[MANTISSA_SECOND_BYTE_INDEX] as u64) << 8;
    mantissa += (header_bytes[MANTISSA_FIRST_BYTE_INDEX] as u64) << 16;

    (exp, mantissa)
}

pub fn compute_threshold(exp: u32, mantissa: u64) -> [bool; 256] {
    let mut threshold_bits = [false; 256];

    for i in 0..256 {
        if (i as u32) < 256 - (exp * 8) && mantissa as u128 & (1u128 << (255u128 - (exp * 8) as u128 - i as u128)) != 0 {
            threshold_bits[i] = true;
        }
    }

    threshold_bits
}

pub fn compute_work(threshold_bits: [bool; 256]) -> BigUint {
    let mut acc = BigUint::new(vec![1]);
    let mut denominator = BigUint::new(vec![0]);
    for i in 0..256 {
        if threshold_bits[255 - i] {
            denominator.add_assign(acc.clone());
        }
        acc.mul_assign(BigUint::new(vec![2]));
    }
    let numerator = acc;

    numerator / denominator
}

pub fn bits_to_bytes32(bits: [bool; 256]) -> [u8; 32] {
    let mut bytes = [0; 32];
    for i in 0..256 {
        bytes[i / 8] |= u8::from(bits[i]) << (7 - i % 8);
    }
    bytes
}