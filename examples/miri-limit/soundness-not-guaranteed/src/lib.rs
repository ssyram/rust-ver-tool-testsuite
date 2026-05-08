//! Miri limitation: Miri cannot guarantee soundness — it only checks one
//! specific execution, not all possible callers.
//!
//! Source: https://github.com/rust-lang/miri/blob/master/README.md
//! "Miri fundamentally cannot ensure that your code is *sound*.
//! [Soundness] is the property of never causing undefined behavior when
//! invoked from arbitrary safe code, even in combination with other sound
//! code.  In contrast, Miri can just tell you if *a particular way of
//! interacting with your code* (e.g., a test suite) causes any undefined
//! behavior *in a particular execution* (of which there may be many, e.g.
//! when concurrency or other forms of non-determinism are involved).
//! When Miri finds UB, your code is definitely unsound, but when Miri does
//! not find UB, then you may just have to test more inputs or more possible
//! non-deterministic choices."
//!
//! Triggered aspect: the safe public API `safe_wrapper_hides_ub` is unsound
//! — it contains latent UB that only manifests for one specific input value
//! (length = 0 → empty slice → `get_unchecked(0)` is OOB).  A test that
//! never passes length = 0 will cause Miri to report no errors, even though
//! the function is unsound.  Miri has no way to discover the bad input on
//! its own; it needs an explicit test harness exercising that path.

/// Returns the first byte of a byte slice, using unchecked indexing.
///
/// # Safety contract (violated)
/// The public signature accepts any `&[u8]`, including empty slices, which
/// makes the function **unsound**: calling it with an empty slice is valid
/// safe Rust that nonetheless triggers undefined behavior.
///
/// Miri will not detect the UB unless a test explicitly passes an empty
/// slice.  A test suite that only passes non-empty slices will be clean under
/// Miri, falsely suggesting the code is safe.
pub fn safe_wrapper_hides_ub(data: &[u8]) -> u8 {
    // SAFETY: intended to be called only with non-empty slices, but this
    // precondition is not enforced by the type system or the public API.
    unsafe { *data.get_unchecked(0) }
}

pub fn trigger_safe_wrapper_hides_ub() {
    let _ = safe_wrapper_hides_ub(&[0u8, 1, 2, 3]);
}
