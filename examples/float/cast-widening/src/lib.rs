/// f32 → f64 widening cast (always lossless) and f64 → f32 narrowing cast (potentially lossy);
/// the narrowing direction can silently lose mantissa bits — a key divergence point for tools
/// that model float precision precisely vs. those that treat all floats uniformly.
pub fn float_cast_widening() {
    // f32 -> f64: exact, no information loss
    let small: f32 = 1.5_f32;
    let wide: f64 = small as f64;  // exactly 1.5
    let _back_narrow: f32 = wide as f32; // lossless round-trip for this value

    // f32 with many mantissa bits: widening is exact
    let precise: f32 = 0.1_f32;
    let _precise_wide: f64 = precise as f64; // ~0.100000001490116 — not 0.1 exactly

    // f64 -> f32 narrowing: 64-bit value that cannot round-trip
    let double_val: f64 = 1.0000000000000002_f64; // just above 1.0 in f64
    let _narrowed: f32 = double_val as f32;        // rounds to 1.0_f32 (loses LSB)

    // special values preserve through cast
    let inf_f32: f32 = f32::INFINITY;
    let _inf_f64: f64 = inf_f32 as f64; // still +inf

    let nan_f32: f32 = f32::NAN;
    let _nan_f64: f64 = nan_f32 as f64; // still NaN
}
