//! Tests: num_integer::Integer trait: gcd, lcm, extended_gcd on BigInt.
//! Deps: num-bigint + num-integer (both pure Rust, no C deps).
//! Operations: Integer::gcd, Integer::lcm, Integer::extended_gcd,
//!             Integer::div_floor, Integer::mod_floor.

use num_bigint::BigInt;
use num_integer::Integer;

pub fn num_integer_gcd() {
    let a = BigInt::from(48_i64);
    let b = BigInt::from(18_i64);

    // gcd and lcm via Integer trait on BigInt
    let g = a.gcd(&b);
    let l = a.lcm(&b);

    // extended gcd: returns (gcd, x, y) s.t. a*x + b*y = gcd
    let ext = a.extended_gcd(&b);
    // ext.gcd == g, ext.x * a + ext.y * b == ext.gcd

    // div_floor / mod_floor (floor division)
    let big_a = BigInt::from(-7_i64);
    let big_b = BigInt::from(3_i64);
    let _df = big_a.div_floor(&big_b); // -3
    let _mf = big_a.mod_floor(&big_b); //  2

    // is_multiple_of
    let twelve = BigInt::from(12_i64);
    let three  = BigInt::from(3_i64);
    let _mult  = twelve.is_multiple_of(&three);

    let _ = (g, l, ext);
}
