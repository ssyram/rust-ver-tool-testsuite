// Creusot limitation: thread-local storage access is not supported.
// Accessing a `thread_local!` variable produces a `Rvalue::ThreadLocalRef`
// node in MIR.  Creusot's statement translator lists this alongside
// `CopyForDeref` and `WrapUnsafeBinder` as an unsupported Rvalue, causing
// a crash with "MIR code used an unsupported Rvalue".
//
// Source: creusot/src/translation/function/statement.rs
//         Rvalue::ThreadLocalRef(_) => crash_and_error("MIR code used an unsupported Rvalue")
//
// cargo check: passes (thread_local! is stable Rust)
// Creusot: crashes with "MIR code used an unsupported Rvalue ThreadLocalRef(...)"

use std::cell::Cell;

thread_local! {
    static COUNTER: Cell<u32> = Cell::new(0);
}

pub fn read_thread_local() -> u32 {
    COUNTER.with(|c| c.get())
}
