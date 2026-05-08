/// Integer ↔ float casts: i32 → f32 precision boundary, f64 → i64 truncation (toward zero),
/// and the Rust-specific saturating cast for out-of-range / NaN → int (stable since 1.45).
/// Tools diverge sharply on the saturating semantics and on whether large-int→f32 is exact.
pub fn float_cast_int() {
    // i32 -> f32: values beyond 2^24 lose precision
    let exact: i32 = 16_777_216; // 2^24, exactly representable in f32
    let _f_exact: f32 = exact as f32;

    let inexact: i32 = 16_777_217; // 2^24 + 1, rounds in f32
    let _f_inexact: f32 = inexact as f32; // may equal 16_777_216.0

    // f64 -> i64: truncation toward zero
    let pos: f64 = 2.9_f64;
    let _trunc_pos: i64 = pos as i64; // 2

    let neg: f64 = -2.9_f64;
    let _trunc_neg: i64 = neg as i64; // -2 (toward zero, not floor)

    // saturating cast semantics (Rust 1.45+): out-of-range clamps, NaN -> 0
    let big: f64 = 1.0e18_f64;
    let _sat_big: i32 = big as i32; // i32::MAX (saturated)

    let nan: f64 = f64::NAN;
    let _sat_nan: i32 = nan as i32; // 0

    let neg_inf: f64 = f64::NEG_INFINITY;
    let _sat_neg_inf: i32 = neg_inf as i32; // i32::MIN (saturated)
}
