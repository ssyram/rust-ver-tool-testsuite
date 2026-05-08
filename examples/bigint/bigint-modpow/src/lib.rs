//! Tests: BigUint modular exponentiation and large modular arithmetic.
//! Dep: num-bigint (pure Rust, no C deps).
//! Operations: BigUint::modpow, %, pow (via std Pow trait).

use num_bigint::BigUint;
use std::str::FromStr;

pub fn bigint_modpow() {
    // modpow: base^exp mod modulus (RSA-like sizes)
    let base    = BigUint::from_str("123456789012345678901234567890").unwrap();
    let exp     = BigUint::from(65537_u64);
    let modulus = BigUint::from_str("999999999999999999999999999999999999999").unwrap();

    let result = base.modpow(&exp, &modulus);

    // plain rem on big numbers
    let big_a = BigUint::from_str("100000000000000000000000000000000").unwrap();
    let big_m = BigUint::from(1_000_000_007_u64);
    let _rem  = &big_a % &big_m;

    // small pow via num-bigint's Pow impl
    let two  = BigUint::from(2_u32);
    let p100 = num_bigint::BigUint::pow(&two, 100u32); // 2^100

    let _ = (result, p100);
}
