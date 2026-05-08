//! Kani limitation: calls to foreign (C FFI) functions lack a Rust body for
//! Kani to analyse, so they are treated as uninterpreted / returning an
//! arbitrary value, making proofs that rely on their postconditions unsound.
//!
//! Source: https://model-checking.github.io/kani/undefined-behaviour.html
//! "Kani fails to verify certain examples because its FFI support requires
//! improvements."
//! Also implied by the function-stubbing blog post:
//! https://model-checking.github.io/kani-verifier-blog/2023/02/28/kani-internship-projects-2022-stubbing.html
//! — stubbing was introduced precisely because FFI bodies are opaque to Kani.
//!
//! Triggered aspect: `abs` is declared in an `extern "C"` block; Kani has no
//! GOTO-program for the C implementation.  A harness asserting `abs(-3) == 3`
//! would be reported as unverifiable (or falsified) because Kani models the
//! return value as non-deterministic rather than executing the C body.

extern "C" {
    /// Absolute value from the C standard library.
    fn abs(n: i32) -> i32;
}

/// Wraps the C `abs` function.  Under Kani the foreign body is opaque and
/// the return value is treated as an arbitrary `i32`, so any property
/// depending on the correct behaviour of `abs` cannot be verified.
pub fn call_libc_abs(x: i32) -> i32 {
    // SAFETY: `abs` is a standard C function with well-defined behaviour for
    // all `i32` values except `i32::MIN` (where UB is possible in C, but
    // that edge case is orthogonal to the FFI-opacity limitation).
    unsafe { abs(x) }
}

pub fn trigger_call_libc_abs() {
    let _ = call_libc_abs(-7);
}
