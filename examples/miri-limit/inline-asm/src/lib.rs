//! Miri limitation: inline assembly is only partially supported.
//!
//! Source: https://github.com/rust-lang/miri/blob/master/README.md
//! "Miri only supports very few AVX512 intrinsics at the moment."
//! More broadly, arbitrary `asm!` blocks that use instructions Miri has no
//! interpreter shim for produce:
//!   error: unsupported operation: inline assembly is not supported
//!
//! Miri has added limited support for `asm!` on x86_64/aarch64 for a small
//! set of instructions (e.g. `cpuid`, `nop`), but any instruction outside that
//! set — such as `rdrand` for reading the hardware RNG — is rejected at
//! interpretation time.
//!
//! Triggered aspect: `asm!("rdrand {0}", ...)` invokes a hardware-RNG
//! instruction for which Miri has no emulation, causing it to report
//! "unsupported operation: inline assembly is not supported" or an equivalent
//! error naming the specific instruction.

/// Reads one 64-bit random value from the hardware RNG via the `rdrand`
/// instruction on x86_64.  Returns `None` when the instruction signals
/// failure (CF=0), or when compiled for a non-x86_64 target.
pub fn add_via_inline_asm(a: u32, b: u32) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        let result: u32;
        // SAFETY: operands are in caller-controlled registers; the `add`
        // instruction does not touch memory.
        unsafe {
            core::arch::asm!(
                "add {0:e}, {1:e}",
                inout(reg) a => result,
                in(reg) b,
                options(pure, nomem, nostack),
            );
        }
        result
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        a.wrapping_add(b)
    }
}

pub fn trigger_add_via_inline_asm() {
    let _ = add_via_inline_asm(1, 2);
}
