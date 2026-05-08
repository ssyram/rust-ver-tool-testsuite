//! Kani limitation: inline assembly (`asm!` and `global_asm!`) is not supported.
//!
//! Source: https://model-checking.github.io/kani/rust-feature-support.html
//! "Assembly: Kani does not support assembly code for now. There are two kinds
//! of assembly code that Rust supports: inline assembly (asm!) and global
//! assembly (global_asm!). Tracking issues: #2, #316."
//!
//! Triggered aspect: the function body contains an `asm!` invocation.
//! Kani encounters the inline assembly block during GOTO-program generation
//! and emits an `assert(false)` followed by `assume(false)`, causing the
//! surrounding proof harness to be immediately unreachable rather than
//! verifying the intended property.

/// Adds two `u64` values using an `x86_64` inline assembly instruction.
/// On any other target the no-op fallback is used, but the `asm!` form
/// remains in the source so that Kani's limitation is always triggered
/// when it compiles this crate.
pub fn add_via_asm(a: u64, b: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        let result: u64;
        // SAFETY: `add` does not write to memory; inputs/output are in
        //         caller-controlled registers.
        unsafe {
            core::arch::asm!(
                "add {0}, {1}",
                inout(reg) a => result,
                in(reg) b,
                options(pure, nomem, nostack),
            );
        }
        result
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        a + b
    }
}

pub fn trigger_add_via_asm() {
    let _ = add_via_asm(3, 4);
}
