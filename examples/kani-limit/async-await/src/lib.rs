//! Kani limitation: `async`/`await` (concurrent futures) are not supported.
//!
//! Source: https://model-checking.github.io/kani/rust-feature-support.html
//! "Await expressions (8.2.18) — No. Concurrent features are currently out of
//! scope for Kani. Kani emits a warning and compiles the code as if it were
//! sequential, which can lead to unsound results."
//!
//! Also: "Data Races (15.3) — No. Concurrency verification remains an open
//! research problem."
//!
//! Triggered aspect: the entry point calls `.await` on an `async fn`.
//! Kani lowers the async state machine into a sequential stub rather than
//! modelling the actual suspension/resumption semantics, meaning that
//! properties that depend on interleaving or on the executor's scheduling
//! cannot be verified.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

async fn async_add(a: u32, b: u32) -> u32 {
    a + b
}

/// Drives a simple async computation to completion using a minimal
/// hand-rolled executor — no external runtime dependency needed.
/// The presence of `.await` in the async body is the feature that
/// Kani flags as out-of-scope for concurrent verification.
pub fn run_async_add() -> u32 {
    // Minimal no-op waker so we can poll the future without tokio/async-std.
    fn clone_raw(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }
    fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_raw, noop, noop, noop);

    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    // SAFETY: vtable functions are all no-ops; the waker is never stored.
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    let mut fut = Box::pin(async_add(1, 2));
    loop {
        match Pin::as_mut(&mut fut).poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {}
        }
    }
}
