//! Charon limitation: branching control flow during `Box` allocation and
//! initialization is not supported.
//!
//! Source: transform/resugar/reconstruct_boxes.rs – "Could not reconstruct
//! `Box` initialization; branching during `Box` initialization is not
//! supported."
//!
//! Rustc lowers `Box::new(x)` into a two-step sequence:
//!   1. `exchange_malloc(...)` – raw allocation (`ShallowInitBox`)
//!   2. write the value into the allocated slot
//! Charon's box-reconstruction pass expects these two steps to appear in the
//! same straight-line block.  When the value expression involves an early
//! return (or any other branch) between allocation and initialization – as the
//! `vec!` macro expansion does here – the pass cannot find the matching write
//! and raises a hard error.
//!
//! Tracking: https://github.com/AeneasVerif/charon/issues/393
//! Confirmed by: charon test file tests/ui/issue-393-shallowinitbox.rs
//! Expected error: "Could not reconstruct `Box` initialization; branching
//! during `Box` initialization is not supported."

/// Helper that can abort early; its return value is deliberately opaque to the
/// optimizer so that the branch in `vec_with_early_return` survives into MIR.
#[inline(never)]
fn maybe_byte() -> Option<u8> {
    // Side-effectful to defeat constant folding.
    let x: u8 = std::hint::black_box(42u8);
    if x > 200 { None } else { Some(x) }
}

/// Builds a `Vec<u8>` whose element comes from a fallible helper, triggering
/// the ShallowInitBox + branching pattern that Charon cannot reconstruct.
///
/// The `vec!` macro desugars to an `exchange_malloc` + `ShallowInitBox` pair.
/// The element expression uses an early-return `?` operator, so there is a
/// branch between the allocation and the write.  Charon's
/// `reconstruct_boxes` pass cannot find the matching assignment and emits:
/// "Could not reconstruct `Box` initialization; branching during `Box`
/// initialization is not supported."
pub fn vec_with_early_return() -> Option<Vec<u8>> {
    let v = vec![maybe_byte()?];
    Some(v)
}
