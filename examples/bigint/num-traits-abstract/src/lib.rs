//! Tests: num_traits trait abstractions (Zero, One, Num, Pow, Signed, Unsigned).
//! Deps: num-traits + num-bigint (pure Rust, no C deps).
//! Operations: Zero::zero/is_zero, One::one, Num::from_str_radix,
//!             Pow::pow, Signed::abs/is_positive/is_negative,
//!             cast/checked_cast via num_traits::cast.

use num_bigint::{BigInt, BigUint};
use num_traits::{Zero, One, Num, Pow, Signed, CheckedAdd, CheckedMul};

/// Generic checked-add via CheckedAdd trait bound.
fn generic_checked_add<T: CheckedAdd>(a: &T, b: &T) -> Option<T> {
    a.checked_add(b)
}

/// Generic checked-mul via CheckedMul trait bound.
fn generic_checked_mul<T: CheckedMul>(a: &T, b: &T) -> Option<T> {
    a.checked_mul(b)
}

/// Generic accumulation: sum 1..=n using Zero + One + Add.
fn sum_to<T>(n: u32) -> T
where
    T: Zero + One + for<'a> std::ops::Add<&'a T, Output = T> + Clone,
{
    let mut acc = T::zero();
    let one = T::one();
    for _ in 0..n {
        acc = acc + &one;
    }
    acc
}

pub fn num_traits_abstract() {
    // Zero / One on BigInt
    let z = BigInt::zero();
    let _is_z = z.is_zero();
    let o = BigInt::one();

    // Num::from_str_radix — hex string to BigUint
    let hex = BigUint::from_str_radix("DEADBEEF", 16).unwrap();

    // Pow on BigUint
    let two: BigUint = BigUint::one() + BigUint::one();
    let p32: BigUint = Pow::pow(two.clone(), 32u32);

    // Signed methods on BigInt
    let neg = BigInt::from(-99_i64);
    let _abs = neg.abs();
    let _pos = neg.is_positive();
    let _neg_flag = neg.is_negative();

    // CheckedAdd / CheckedMul via generic trait-bounded helpers
    let big = BigInt::from(i64::MAX);
    let _chk_add = generic_checked_add(&big, &o);
    let _chk_mul = generic_checked_mul(&big, &BigInt::from(2_i64));

    // generic sum
    let s: BigInt = sum_to(10);

    let _ = (hex, p32, s);
}
