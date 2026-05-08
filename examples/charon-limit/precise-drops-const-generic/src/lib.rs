//! Charon limitation: when `--precise-drops` is enabled, a struct that
//! combines a const-generic array field with a `Box`-bearing field triggers a
//! rustc internal compiler error during drop-glue retrieval.
//!
//! Source: translate_drops.rs – "rustc panicked while retrieving drop glue.
//! This is known to happen with `--precise-drops`".
//! Also: charon test file tests/ui/simple/drop-glue-with-const-generic.rs
//!
//! Tracking: https://github.com/AeneasVerif/charon/issues/918
//! Status: open as of v0.1.184
//!
//! Root cause: rustc's drop-elaboration pass is not written to handle const
//! generics in param-env contexts.  When Charon requests precise drop glue for
//! a type like `PortableHash<K>` (which contains `[KeccakState; K]`), rustc
//! cannot locate the const parameter `K` in the local param-env and panics.
//! Charon catches the panic and reports it as a hard error.
//!
//! This entry requires Charon to be invoked with `--precise-drops`.
//! Without that flag the drop-glue query is not issued and translation
//! succeeds; the limitation is specific to that mode.

/// A zero-sized type whose presence makes the array field non-trivially
/// destructible (it gets a `KeccakState` drop impl).
pub struct KeccakState;

/// A struct with a const-generic array of `KeccakState` values and an owned
/// `Box<u32>` that forces the compiler to generate non-trivial drop glue.
/// When Charon runs with `--precise-drops`, requesting this struct's drop
/// implementation causes rustc to panic because const generic `K` cannot be
/// found in the elaborated param-env.
pub struct PortableHash<const K: usize> {
    pub shake128_state: [KeccakState; K],
    pub make_non_trivial: Box<u32>,
}

/// Constructs a `PortableHash<2>` and immediately drops it so that Charon
/// must request the struct's drop glue when running with `--precise-drops`.
pub fn consume_portable_hash() {
    let _h = PortableHash {
        shake128_state: [KeccakState, KeccakState],
        make_non_trivial: Box::new(0u32),
    };
    // _h is dropped here, triggering drop-glue elaboration.
}
