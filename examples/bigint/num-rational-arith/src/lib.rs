//! Tests: num_rational::BigRational (arbitrary-precision rational numbers).
//! Deps: num-rational + num-bigint + num-traits (all pure Rust, no C deps).
//! Operations: new, add, sub, mul, div, cmp, numer/denom, reduced form,
//!             recip, abs, signum via num_traits.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero, Signed};

pub fn num_rational_arith() {
    // construct 3/4 and 5/6
    let a = BigRational::new(BigInt::from(3), BigInt::from(4));
    let b = BigRational::new(BigInt::from(5), BigInt::from(6));

    // arithmetic (always auto-reduces)
    let sum  = &a + &b;   // 3/4 + 5/6 = 19/12
    let diff = &a - &b;   // 3/4 - 5/6 = -1/12
    let prod = &a * &b;   // 15/24 = 5/8
    let quot = &a / &b;   // 18/20 = 9/10

    // zero and one
    let _zero = BigRational::zero();
    let _one  = BigRational::one();

    // comparison
    let _lt = a < b;
    let _eq = sum == BigRational::new(BigInt::from(19), BigInt::from(12));

    // numer / denom access
    let _n = prod.numer();
    let _d = prod.denom();

    // reciprocal
    let _recip = quot.recip();

    // abs / signum from Signed
    let neg = -&a;
    let _abs = neg.abs();
    let _sig = neg.signum();

    let _ = (sum, diff);
}
