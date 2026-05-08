// Creusot limitation: dyn Trait support is minimal and experimental.
// Only dyn Debug and dyn Write are "logically dyn-compatible"; all other
// trait objects hit "forbidden dyn type" at translation time.
//
// Source: creusot/src/backend/clone_map/elaborator.rs – is_logically_dyn_compatible_trait
//         Issue #2071: `forbidden dyn type: dyn core::error::Error`
//
// cargo check: passes (plain Rust, dyn Display is valid Rust)
// Creusot: crashes with "forbidden dyn type: dyn core::fmt::Display
//          (dyn support is currently minimal, please open an issue)"

use std::fmt::Display;

pub fn call_dyn_display(val: &dyn Display) -> String {
    format!("{}", val)
}

pub fn trigger_call_dyn_display() {
    let _ = call_dyn_display(&42i32);
}
