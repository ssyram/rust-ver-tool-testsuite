// Hax limitation: `unsafe` blocks are rejected unconditionally.
//
// Hax error: HAX0000 (UnsafeBlock)
// "Unsafe blocks are not allowed."
//
// Source: hax-types/src/diagnostics/mod.rs, Kind::UnsafeBlock = 0
// Also: docs/manual/fstar/quick_start.md states
//   "hax doesn't support every Rust constructs, `unsafe` code,
//    or complicated mutation scheme."
// The diagnostics Display impl produces: "Unsafe blocks are not allowed."

pub fn hax_limit_unsafe_block() -> u32 {
    let a = 100u32;
    let b = 200u32;
    // Any `unsafe { ... }` block triggers HAX0000 regardless of content.
    unsafe { a.unchecked_add(b) }
}
