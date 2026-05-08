/// i64 wrapping_* arithmetic: modulo 2^64 semantics on a 64-bit signed type.
/// Tests whether verifiers handle wrapping ops on the full 64-bit width correctly,
/// including negative results from unsigned-style wrap-around.
pub fn int_width_wrapping_i64() {
    let max: i64 = i64::MAX; // 2^63 - 1
    let min: i64 = i64::MIN; // -2^63

    // max + 1 wraps to min
    let _w1: i64 = max.wrapping_add(1_i64);   // -2^63

    // min - 1 wraps to max
    let _w2: i64 = min.wrapping_sub(1_i64);   // 2^63 - 1

    // Signed multiplication overflow
    let _w3: i64 = max.wrapping_mul(2_i64);   // -2 (wraps)

    // wrapping_neg: negation of MIN
    let _w4: i64 = min.wrapping_neg();         // min (no positive counterpart)

    // wrapping_abs on MIN
    let _w5: i64 = min.wrapping_abs();         // min (wraps back)
}
