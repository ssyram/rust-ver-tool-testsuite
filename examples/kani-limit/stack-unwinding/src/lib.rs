//! Kani limitation: stack unwinding (the panic-unwind strategy) is not
//! supported; only the `panic = "abort"` strategy works correctly.
//!
//! Source: https://model-checking.github.io/kani/rust-feature-support.html
//! "Stack Unwinding — No. Kani does not support stack unwinding. This means
//! that Kani currently does not support programs that use the unwinding panic
//! strategy. This can cause resource leaks or data inconsistencies. Issue #692
//! tracks adding unwinding support."
//!
//! Triggered aspect: the function calls `std::panic::catch_unwind`, which
//! relies on the unwinding panic strategy to catch a panic propagating up
//! the call stack.  Under Kani's abort-only model the catch block is never
//! reachable: a panicking sub-call aborts the program instead of unwinding,
//! so any property that depends on `catch_unwind` returning `Err` cannot be
//! verified.

/// Attempts integer division, catching a potential divide-by-zero panic via
/// `catch_unwind`.  The recovery path (`Err` arm) is unreachable under Kani
/// because Kani does not model stack unwinding.
pub fn divide_with_recovery(numerator: i32, denominator: i32) -> i32 {
    let result = std::panic::catch_unwind(|| numerator / denominator);
    match result {
        Ok(v) => v,
        Err(_) => -1, // recovery path — dead under Kani's abort strategy
    }
}

pub fn trigger_divide_with_recovery() {
    let _ = divide_with_recovery(10, 2);
}
