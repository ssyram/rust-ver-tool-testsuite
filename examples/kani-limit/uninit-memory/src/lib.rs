//! Kani limitation: detection of reads from uninitialized memory is
//! experimental and only available behind `-Z uninit-checks`; it is not
//! enabled in standard verification runs.
//!
//! Source: https://model-checking.github.io/kani/rust-feature-support.html
//! "Uninitialized Memory — Experimental support via `-Z uninit-checks` option
//! (issue #3300); partial functionality."
//!
//! Also: https://model-checking.github.io/kani/undefined-behaviour.html
//! "Kani won't complain if you transmute an invalid value to a Rust type."
//!
//! Triggered aspect: a `MaybeUninit<u8>` is allocated but never written;
//! `assume_init` is called, which is undefined behaviour under Rust's memory
//! model.  Without `-Z uninit-checks` Kani silently treats the uninitialised
//! byte as an arbitrary concrete value and does not flag the UB, leaving the
//! bug invisible to the default verification pass.

use std::mem::MaybeUninit;

/// Reads one byte from a `MaybeUninit<u8>` that was never initialised.
/// This is UB per the Rust reference; the point of this entry is that
/// Kani's default mode does NOT detect it — only the experimental
/// `-Z uninit-checks` flag would flag this as a violation.
pub fn read_uninit_byte() -> u8 {
    let x: MaybeUninit<u8> = MaybeUninit::uninit();
    // SAFETY (intentionally violated): calling `assume_init` on an
    // uninitialised value is unsound; this is the pattern Kani's
    // experimental checker is designed to catch.
    unsafe { x.assume_init() }
}
