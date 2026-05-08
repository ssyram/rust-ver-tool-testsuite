//! Miri limitation: calling a foreign (C) function that has no Miri shim
//! produces an "unsupported operation" error.
//!
//! Source: https://github.com/rust-lang/miri/blob/master/README.md
//! "Miri runs the program as a platform-independent interpreter, so the
//! program has no access to most platform-specific APIs or FFI."
//!
//!   error: unsupported operation: can't call foreign function: bind
//!     = help: this is likely not a bug in the program; it indicates that the
//!             program performed an operation that Miri does not support
//!
//! Miri ships shims for a curated set of libc functions (write, read, open,
//! getenv, …) but most C library functions are absent.  `getpid` is a
//! commonly used POSIX function that has historically not been shimmed in all
//! Miri configurations, serving as a minimal demonstration of the pattern.
//!
//! Triggered aspect: declaring `getpid` as an `extern "C"` function and
//! calling it causes Miri to look up a shim; if none is found it reports
//! "unsupported operation: can't call foreign function: getpid".  (`cargo
//! check` passes because the declaration is valid Rust; only Miri's
//! interpreter rejects it at runtime.)

extern "C" {
    /// POSIX `getpid(2)` — returns the process ID of the calling process.
    ///
    /// Miri does not shim this function on all targets.  On targets where the
    /// shim is absent the call triggers:
    ///   error: unsupported operation: can't call foreign function: getpid
    fn getpid() -> i32;
}

/// Calls the C `getpid` function and returns the result.
///
/// Under Miri this may hit an unsupported-operation error if the `getpid`
/// shim is absent.  The function compiles cleanly under `cargo check` because
/// the `extern "C"` declaration is valid Rust.
pub fn call_unshimmed_foreign_fn() -> i32 {
    // SAFETY: `getpid` takes no arguments, performs no memory operations
    // visible from Rust, and always succeeds.
    unsafe { getpid() }
}
