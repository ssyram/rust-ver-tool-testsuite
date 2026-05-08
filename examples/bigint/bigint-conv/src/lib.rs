//! Tests: BigInt <-> primitive integer conversions.
//! Deps: num-bigint + num-traits (pure Rust, no C deps).
//! Operations: ToBigInt::to_bigint(), ToPrimitive::to_i64/to_u64,
//!             BigInt::from_bytes_be / to_bytes_be, BigInt::from_signed_bytes_le.

use num_bigint::{BigInt, BigUint, Sign, ToBigInt};
use num_traits::ToPrimitive;

pub fn bigint_conv() {
    // primitive -> BigInt via ToBigInt trait
    let from_i32: BigInt = 42_i32.to_bigint().unwrap();
    let from_u64: BigInt = 0xDEAD_BEEF_u64.to_bigint().unwrap();

    // BigInt -> primitive via ToPrimitive
    let back_i64: Option<i64>  = from_i32.to_i64();
    let back_u32: Option<u32>  = from_u64.to_u32();

    // byte-level round-trip
    let big = BigInt::from(-123_456_789_i64);
    let (sign, bytes) = big.to_bytes_be();
    let restored = BigInt::from_bytes_be(sign, &bytes);

    // BigUint from bytes
    let ubytes: Vec<u8> = vec![0xFF, 0x00, 0xAB];
    let ubig = BigUint::from_bytes_be(&ubytes);
    let _back_bytes = ubig.to_bytes_be();

    // sign extraction
    let negative = BigInt::from(-1_i64);
    let _sign = negative.sign(); // Sign::Minus

    let _ = (back_i64, back_u32, restored, Sign::NoSign);
}
