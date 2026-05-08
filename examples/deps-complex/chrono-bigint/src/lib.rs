//! Tests: chrono + num-bigint + num-traits cross-crate type conversion.
//! Extracts nanosecond timestamps from chrono::DateTime<Utc>, converts them
//! to num-bigint::BigInt, performs arbitrary-precision arithmetic, and
//! converts back via num-traits::ToPrimitive.
//! Interaction: chrono's timestamp_nanos_opt() returns i64; we widen to BigInt
//! via From<i64>; arithmetic is done in BigInt space; result is narrowed back
//! via num_traits::ToPrimitive::to_i64(). This exercises cross-crate numeric
//! type coercions that verifiers must follow through three crate boundaries.

use chrono::{DateTime, TimeZone, Utc};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

pub fn chrono_bigint() {
    let t0: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
    let t1: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();

    // convert timestamps to nanoseconds as BigInt
    let ns0 = BigInt::from(t0.timestamp_nanos_opt().unwrap());
    let ns1 = BigInt::from(t1.timestamp_nanos_opt().unwrap());

    // arbitrary-precision interval arithmetic
    let interval_ns = &ns1 - &ns0;
    let ns_per_day = BigInt::from(86_400_000_000_000_i64);
    let days = &interval_ns / &ns_per_day;
    let remainder_ns = &interval_ns % &ns_per_day;

    // narrow back to i64 via num-traits
    let days_i64 = days.to_i64().unwrap();
    let rem_i64 = remainder_ns.to_i64().unwrap();

    // reconstruct approximate DateTime to verify round-trip sanity
    let reconstructed_ns = ns0 + BigInt::from(days_i64) * &ns_per_day + BigInt::from(rem_i64);
    let reconstructed_i64 = reconstructed_ns.to_i64().unwrap();
    let _rt: DateTime<Utc> = DateTime::from_timestamp_nanos(reconstructed_i64);
}
