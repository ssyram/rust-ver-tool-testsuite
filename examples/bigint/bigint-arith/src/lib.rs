//! Tests: BigInt arbitrary-precision arithmetic and comparison.
//! Dep: num-bigint (pure Rust, no C deps).
//! Operations: add, sub, mul, div, rem, neg, cmp via BigInt.

use num_bigint::BigInt;
use std::str::FromStr;

pub fn bigint_arith() {
    // construct from literal and from string
    let a = BigInt::from(1_000_000_000_i64);
    let b = BigInt::from_str("999999999999999999999999999").unwrap();

    // arithmetic operators
    let sum = &a + &b;
    let diff = &b - &a;
    let product = BigInt::from(123_456i64) * BigInt::from(987_654i64);
    let quot = &b / &a;
    let rem = &b % &a;

    // comparisons
    let _gt = b > a;
    let _eq = sum == diff + &a + &a;

    // negation
    let neg = -BigInt::from(42i64);
    let _back = -neg;

    // suppress unused warnings
    let _ = (product, quot, rem);
}
