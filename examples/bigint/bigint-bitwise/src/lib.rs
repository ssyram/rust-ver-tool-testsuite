//! Tests: BigUint bitwise operations: AND, OR, XOR, left/right shift.
//! Dep: num-bigint (pure Rust, no C deps).
//! Operations: &, |, ^, <<, >> on BigUint values.

use num_bigint::BigUint;

pub fn bigint_bitwise() {
    let a = BigUint::from(0b1100_1010_u32);
    let b = BigUint::from(0b1010_0110_u32);

    // bitwise AND, OR, XOR
    let _and = &a & &b;
    let _or  = &a | &b;
    let _xor = &a ^ &b;

    // left and right shift
    let shifted_left  = &a << 4usize;
    let shifted_right = &shifted_left >> 4usize;

    // large value: shift a 256-bit-wide number
    let big = BigUint::from(u128::MAX);
    let _wide = &big << 128usize;

    let _ = shifted_right;
}
