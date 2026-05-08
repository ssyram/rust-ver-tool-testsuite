/// Integer <-> float conversions: i64->f64 lossless boundary, i32->f64,
/// f64->i32 truncation (round-toward-zero), and the saturating cast behavior
/// when the float value is out of range for the target integer type.
/// Verifiers that don't model IEEE 754 round-to-nearest and integer truncation
/// will diverge here, especially on boundary values near 2^53 (f64 mantissa limit).
pub fn int_width_cast_float_int() {
    // i32 -> f64: always exact (i32 fits in f64's 52-bit mantissa)
    let a: i32 = i32::MAX;         // 2^31 - 1
    let _f1: f64 = a as f64;       // exact: 2147483647.0

    // i64 -> f64: lossless only up to 2^53
    let exact: i64 = (1_i64 << 53) - 1; // 2^53 - 1, last exactly representable
    let _f2: f64 = exact as f64;         // exact

    let inexact: i64 = (1_i64 << 53) + 1; // 2^53 + 1, rounds to 2^53 + 2 in f64
    let _f3: f64 = inexact as f64;         // rounded — not bit-for-bit exact

    // f64 -> i32: truncation toward zero
    let _i1: i32 = 1.9_f64 as i32;   // 1
    let _i2: i32 = (-1.9_f64) as i32; // -1
    let _i3: i32 = 100.99_f64 as i32; // 100

    // f64 -> i64: truncation
    let _i4: i64 = 1.0e18_f64 as i64; // 1_000_000_000_000_000_000

    // Saturating cast behavior in Rust: f64 out-of-range -> clamp to i32 boundary
    // (using `as` in Rust gives "saturating" behavior since Rust 1.45)
    let _i5: i32 = 1.0e18_f64 as i32;  // i32::MAX (saturates)
    let _i6: i32 = -1.0e18_f64 as i32; // i32::MIN (saturates)

    // u8 -> f32 lossless (all u8 values representable in f32)
    let _f4: f32 = 255_u8 as f32; // 255.0
}
