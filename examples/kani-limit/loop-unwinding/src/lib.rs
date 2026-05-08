//! Kani limitation: loops with an input-dependent (unbounded) iteration count
//! cause non-termination or require an explicit `#[kani::unwind(n)]` bound.
//!
//! Source: https://model-checking.github.io/kani/tutorial-loop-unwinding.html
//! "Kani cannot verify code with 'unbounded' loops — those lacking a constant
//! number of iterations that the verifier can recognize. When the unwinding
//! bound is insufficient, users encounter 'unwinding assertion' failures or
//! non-termination."
//!
//! Triggered aspect: the loop bound is `n`, a runtime value unknown to Kani
//! at verification time.  Without an explicit `#[kani::unwind]` annotation
//! Kani will either diverge (try to unwind indefinitely) or — with a finite
//! default bound — report an "unwinding assertion FAILED" result, meaning
//! the proof is only valid up to that bound and cannot give a universal
//! guarantee over all `n`.

/// Returns the sum `0 + 1 + … + (n-1)`.  The loop iterates `n` times;
/// because `n` is a function parameter Kani treats it as an unbounded
/// symbolic value, triggering the loop-unwinding limitation.
pub fn sum_to_n(n: u64) -> u64 {
    let mut acc: u64 = 0;
    for i in 0..n {
        acc = acc.wrapping_add(i);
    }
    acc
}

pub fn trigger_sum_to_n() {
    let _ = sum_to_n(5);
}
