//! Charon limitation: unsizing a generic type parameter to a `dyn Trait`
//! smart-pointer is not supported.
//!
//! In charon's own test suite, every function that coerces `Box<T>` (where
//! `T` is a generic type variable bound by a trait) to `Box<dyn Trait>` is
//! annotated with `#[charon::opaque]` and the comment "Opaque because we
//! don't support unsize coercions."
//!
//! Source: charon/tests/ui/dyn-trait.rs – the `construct` function:
//!   ```ignore
//!   // Opaque because we don't support unsize coercions.
//!   #[charon::opaque]
//!   fn construct<T: Display + 'static>(x: T) -> Box<dyn Display> { Box::new(x) }
//!   ```
//!
//! When the `#[charon::opaque]` workaround is removed, Charon attempts to
//! translate the `CoerceUnsized` / `Unsize` cast from `Box<T>` to
//! `Box<dyn Display>`, which involves computing vtable metadata for a generic
//! type parameter.  This triggers translation failures in the vtable
//! construction path.
//!
//! Related: https://github.com/AeneasVerif/charon/issues/608 (now closed but
//! generic-T→dyn coercions remain marked opaque in the test suite).

use std::fmt::Display;

/// Generic helper that boxes any `Display` implementor into `Box<dyn Display>`.
///
/// The body performs a `CoerceUnsized` coercion from `Box<T>` to
/// `Box<dyn Display>`.  Because `T` is a type parameter rather than a
/// concrete type, Charon cannot resolve the vtable at translation time and
/// fails.  (With a concrete type like `String`, the same cast succeeds.)
fn wrap_display<T: Display + 'static>(x: T) -> Box<dyn Display> {
    Box::new(x)
}

/// Wraps the integer 42 as a `Box<dyn Display>` via a generic helper.
///
/// The zero-arg entry point drives Charon into `wrap_display::<u32>`, whose
/// body contains the unsupported generic-T-to-dyn-Trait unsizing coercion.
pub fn boxed_display_from_u32() -> Box<dyn Display> {
    wrap_display(42u32)
}
