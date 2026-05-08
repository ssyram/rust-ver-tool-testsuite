//! Charon limitation: unsizing coercions through custom smart pointers such
//! as `Arc<[T; N]>` → `Arc<[T]>` are not supported.
//!
//! Tracking: https://github.com/AeneasVerif/charon/issues/855
//! ("Feature request: Support unsizing behind custom smart pointers")
//! Status: open as of v0.1.184
//!
//! Rust allows coercing `Arc<[T; N]>` to `Arc<[T]>` via the `CoerceUnsized`
//! trait, which `Arc` implements.  This coercion adds length metadata to the
//! fat pointer stored inside `Arc`.  Charon's unsizing metadata computation
//! calls `struct_lockstep_tails_raw` to find the innermost field that changes
//! type, then looks up the unsizing metadata for that field.  For library
//! smart pointers like `Arc` or `Rc` it ends up returning
//! `UnsizingMetadata::Unknown`, which Charon cannot lower to LLBC and causes
//! a translation failure or incorrect output.
//!
//! Note: `Box<[T; N]>` → `Box<[T]>` works because `Box` receives special
//! first-class treatment in Charon; only third-party/std smart pointers that
//! go through the generic `CoerceUnsized` path are affected.

use std::sync::Arc;

/// Coerces a fixed-size array wrapped in `Arc` into a slice wrapped in `Arc`.
///
/// The coercion from `Arc<[u32; 3]>` to `Arc<[u32]>` follows the generic
/// `CoerceUnsized` path, which Charon cannot translate for non-`Box` smart
/// pointers.
pub fn arc_array_to_slice() -> Arc<[u32]> {
    let arr: Arc<[u32; 3]> = Arc::new([10u32, 20, 30]);
    arr
}
