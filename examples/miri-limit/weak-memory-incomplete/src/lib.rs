//! Miri limitation: weak memory emulation is incomplete.
//!
//! Source: https://github.com/rust-lang/miri/blob/master/README.md
//! "Weak memory emulation is not complete: there are legal behaviors that
//! Miri will never produce. However, Miri produces many behaviors that are
//! hard to observe on real hardware, so it can help quite a bit in finding
//! weak memory concurrency bugs. To be really sure about complicated atomic
//! code, use specialized tools such as loom."
//!
//! The C++11 memory model permits a `Relaxed` load to observe a store that is
//! not the most-recent one in program order.  Miri emulates *some* of these
//! stale-read behaviors (controlled by `-Zmiri-disable-weak-memory-emulation`),
//! but the emulation is a probabilistic approximation: it does not exhaustively
//! model every valid execution.  Certain legal weak behaviors — such as a load
//! returning a value from a store that happened arbitrarily far in the past —
//! may never be produced by Miri even across many seeds.
//!
//! Triggered aspect: two threads write and read a shared `AtomicU32` with
//! `Relaxed` ordering.  Real weak hardware (ARM, POWER) may allow the reader
//! to observe a stale value; Miri's incomplete model means it will not always
//! produce every legal outcome even when running with `-Zmiri-many-seeds`.

use std::sync::atomic::{AtomicU32, Ordering};

static SHARED: AtomicU32 = AtomicU32::new(0);

/// Performs a `Relaxed` store then a `Relaxed` load on the same atomic.
///
/// Returns the observed value.  Under the C++11 weak-memory model a
/// concurrent reader on another thread could legally see the old value (0)
/// even after the store of 1 has been issued, but Miri's emulation may never
/// produce that outcome, silently missing the class of bugs that depend on it.
pub fn relaxed_load_may_not_observe_all_stores() -> u32 {
    SHARED.store(1, Ordering::Relaxed);
    SHARED.load(Ordering::Relaxed)
}
