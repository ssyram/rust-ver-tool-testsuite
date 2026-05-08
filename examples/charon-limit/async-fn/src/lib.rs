//! Charon limitation: `async fn` bodies are compiled by rustc into coroutines
//! (state-machine types), and Charon cannot translate coroutine types or
//! coroutine aggregate values.
//!
//! Source: translate_types.rs – `TyKind::Coroutine(..) => raise_error!("Coroutine types are not
//! supported yet")`.  Also translate_bodies.rs – `AggregateKind::Coroutine(..) => raise_error!(
//! "Coroutines are not supported")`.
//!
//! Tracking issue: https://github.com/AeneasVerif/charon/issues/609
//!
//! Triggered aspect: the function body is desugared to a coroutine by rustc, so
//! Charon encounters a `TyKind::Coroutine` when translating the return type and
//! an `AggregateKind::Coroutine` when translating the body, both of which raise
//! hard errors.

/// Returns the constant value 42 asynchronously.
///
/// The `async fn` syntax causes rustc to generate a coroutine state machine
/// whose type Charon cannot translate.  The function is otherwise trivial safe
/// stable Rust and passes `cargo check`.
pub async fn async_forty_two() -> u32 {
    42
}
