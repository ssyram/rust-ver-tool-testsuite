//! Charon limitation: inline assembly blocks (`asm!`) are not supported.
//!
//! Source: translate_bodies.rs – `TerminatorKind::InlineAsm { .. } => raise_error!("Inline
//! assembly is not supported")`.
//!
//! When rustc lowers an `asm!()` invocation it produces an `InlineAsm` MIR
//! terminator.  Charon raises a hard error on every such terminator because
//! assembly semantics have no representation in LLBC.
//!
//! Note: `global_asm!` (module-level assembly) is silently *skipped* by Charon
//! rather than erroring; only inline assembly inside function bodies triggers
//! the failure.

/// Issues a single `nop` instruction via inline assembly on x86-64 targets.
/// On other architectures the body is a no-op that still exercises the asm!
/// invocation through a cfg-guarded fallback so that `cargo check` always
/// passes regardless of the host.
pub fn nop_via_asm() {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: `nop` reads/writes nothing; pure no-op.
        unsafe {
            core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Non-x86 fallback: still includes an asm! so Charon always encounters
        // the InlineAsm terminator on any target.
        unsafe {
            core::arch::asm!("");
        }
    }
}
