//! Charon limitation: dereferencing a generic `Copy` reference inside a
//! closure causes a type-checking failure in Charon's post-transformation pass.
//!
//! Charon bug: https://github.com/AeneasVerif/charon/issues/1120
//! Status: open as of v0.1.184
//!
//! When a closure captures a reference `x: &T` where `T: Copy` and the
//! closure body dereferences `x` (producing a copy of `*x`), the closure's
//! generic parameters include a `Copy` clause bound at a particular De Bruijn
//! level.  Charon's transformation pipeline mis-indexes this clause variable,
//! producing an "incorrect clause var: Bound(1, 0)" error during the final
//! type-checking pass.
//!
//! Expected error: "Type error after transformations: Found incorrect clause
//! var: Bound(1, 0)"

/// Helper with a concrete `Copy` type so that the zero-arg public function
/// below can call into the generic path that triggers the bug.
fn deref_copy_generic<T: Copy>(x: &T) -> T {
    let get = || *x;
    get()
}

/// Invokes the generic helper with `u32`, a `Copy` type, to trigger the
/// mis-indexed clause-variable bug in Charon's post-transformation pass.
pub fn deref_copy_in_closure() -> u32 {
    let val = 42u32;
    deref_copy_generic(&val)
}
