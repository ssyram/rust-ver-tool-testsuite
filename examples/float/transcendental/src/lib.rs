/// Transcendental intrinsics: f64::sqrt and f64::sin; most static verifiers either
/// stub these as uninterpreted functions or refuse to reason about them entirely,
/// making this a hard differentiator between tools with real-arithmetic models and those without.
pub fn float_transcendental() {
    // sqrt: exact for perfect squares
    let four: f64 = 4.0_f64;
    let _sqrt4: f64 = four.sqrt(); // 2.0 exactly

    let two: f64 = 2.0_f64;
    let _sqrt2: f64 = two.sqrt(); // irrational — no finite IEEE 754 representation

    // sqrt of special values
    let _sqrt_neg: f64 = (-1.0_f64).sqrt(); // NaN
    let _sqrt_inf: f64 = f64::INFINITY.sqrt(); // +inf
    let _sqrt_zero: f64 = (0.0_f64).sqrt(); // 0.0

    // sin: well-known values
    let pi: f64 = std::f64::consts::PI;
    let _sin_zero: f64 = (0.0_f64).sin(); // 0.0
    let _sin_pi_half: f64 = (pi / 2.0).sin(); // ~1.0 (not exactly 1.0 due to pi approximation)
    let _sin_pi: f64 = pi.sin(); // ~0.0 (not exactly 0.0)

    // sin of special values
    let _sin_inf: f64 = f64::INFINITY.sin(); // NaN
    let _sin_nan: f64 = f64::NAN.sin();      // NaN
}
