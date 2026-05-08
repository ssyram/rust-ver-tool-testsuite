//! Kani limitation: trigonometric and other transcendental floating-point
//! operations are over-approximated, producing spurious verification failures.
//!
//! Source: https://model-checking.github.io/kani/rust-feature-support/intrinsics.html
//! "Partially Supported — Math Functions: cosf32, cosf64, sinf32, sinf64,
//! exp2f32, exp2f64, expf32, expf64, fmaf32, fmaf64, log10f32, log10f64,
//! log2f32, log2f64, logf32, logf64, powf32, powf64, powif32, powif64,
//! sqrtf32, sqrtf64 — results are over-approximated."
//!
//! Also from https://model-checking.github.io/kani/rust-feature-support.html:
//! "Floating Point — Over-approximation of trigonometric and sqrt functions;
//! may produce spurious errors unrelated to actual execution."
//!
//! Triggered aspect: the function calls `f64::sin` and `f64::cos`, both of
//! which are modelled as non-deterministic (over-approximated) intrinsics in
//! Kani.  A property asserting the Pythagorean identity `sin²θ + cos²θ = 1`
//! would be reported as *violated* even though it holds for all real inputs,
//! because Kani's over-approximation allows the returned values to range over
//! a superset of the true image of sin/cos.

/// Computes `sin²(x) + cos²(x)` — mathematically always 1.0, but Kani's
/// over-approximated intrinsics for `sin`/`cos` mean a proof harness
/// asserting the result equals `1.0` would produce a spurious counterexample.
pub fn check_sin_cos_identity(x: f64) -> f64 {
    let s = x.sin();
    let c = x.cos();
    s * s + c * c
}

pub fn trigger_check_sin_cos_identity() {
    let _ = check_sin_cos_identity(0.5);
}
