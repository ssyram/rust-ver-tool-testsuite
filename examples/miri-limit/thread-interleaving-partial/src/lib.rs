//! Miri limitation: only one thread interleaving is explored per execution.
//!
//! Source: https://github.com/rust-lang/miri/blob/master/README.md
//! "Program execution is non-deterministic when it depends, for example, on
//! where exactly in memory allocations end up, or on the exact interleaving
//! of concurrent threads.  Miri tests one of many possible executions of your
//! program, but it will miss bugs that only occur in a different possible
//! execution.  You can alleviate this to some extent by running Miri with
//! different values for `-Zmiri-seed`, but that will still by far not explore
//! all possible executions."
//!
//! Miri is a single-threaded interpreter that schedules threads itself using
//! a randomized (or fixed) schedule selected by its internal RNG seed.  A
//! data race that only manifests under a specific interleaving (e.g., thread B
//! reads between thread A's two writes) may never be scheduled by Miri's
//! chosen execution order, causing the race to go undetected even though the
//! code is formally racy.
//!
//! Triggered aspect: two threads increment a shared counter without
//! synchronisation.  The race is only observable under certain interleavings;
//! Miri may (or may not) catch it depending on the seed.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// Spawns two threads that each increment a shared `AtomicUsize` counter
/// using non-atomic `load` + `store` (i.e., a read-modify-write gap).
///
/// The classic data race here requires a specific interleaving: both threads
/// must read the same value before either writes back.  Miri explores only
/// one execution path per seed, so the race may not be detected at all, or
/// may only surface when run with many different seeds.
pub fn unsynchronised_counter_race() -> usize {
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = Arc::clone(&counter);
    let t1 = thread::spawn(move || {
        // Deliberate non-atomic RMW: load then store with a potential gap.
        let v = c1.load(Ordering::Relaxed);
        c1.store(v.wrapping_add(1), Ordering::Relaxed);
    });

    let c2 = Arc::clone(&counter);
    let t2 = thread::spawn(move || {
        let v = c2.load(Ordering::Relaxed);
        c2.store(v.wrapping_add(1), Ordering::Relaxed);
    });

    t1.join().unwrap();
    t2.join().unwrap();
    counter.load(Ordering::Relaxed)
}
