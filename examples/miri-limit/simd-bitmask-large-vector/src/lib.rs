//! Miri limitation: most AVX-512 intrinsics are not supported.
//!
//! Source: https://github.com/rust-lang/miri/blob/master/README.md
//! "Miri only supports very few AVX512 intrinsics at the moment."
//!
//! AVX-512 introduced 512-bit SIMD registers and hundreds of new instructions.
//! Miri emulates only a small subset; most AVX-512 intrinsics trigger:
//!   error: unsupported operation: unimplemented intrinsic: `avx512...`
//!
//! The `simd_bitmask` intrinsic is also limited to vectors of at most 64
//! elements (issue #3658); AVX-512 often operates on 64-byte vectors of i8
//! which are exactly at that limit, and wider portable-simd vectors exceed it.
//!
//! Triggered aspect: calling `_mm512_abs_epi8` (AVX-512BW absolute value on
//! 64 x i8 lanes) causes Miri to report an unsupported-operation error because
//! no interpreter shim exists for this intrinsic.  `cargo check` and `rustc`
//! compile the call successfully when the `avx512bw` target feature is active;
//! the limitation is Miri-only.

/// Returns the element-wise absolute value of a 512-bit vector of 64 `i8`
/// lanes using the AVX-512BW `_mm512_abs_epi8` intrinsic.
///
/// Under Miri this hits an unsupported-operation error; on real hardware with
/// AVX-512BW support it executes normally.
#[cfg(target_arch = "x86_64")]
pub fn bitmask_over_64_elements(input: [i8; 64]) -> [i8; 64] {
    #[cfg(target_feature = "avx512bw")]
    unsafe {
        use core::arch::x86_64::*;
        // Load 64 bytes into a __m512i register.
        let v = _mm512_loadu_si512(input.as_ptr() as *const i32);
        // Compute absolute value — not shimmed in Miri.
        let abs = _mm512_abs_epi8(v);
        let mut out = [0i8; 64];
        _mm512_storeu_si512(out.as_mut_ptr() as *mut i32, abs);
        out
    }
    #[cfg(not(target_feature = "avx512bw"))]
    {
        let mut out = input;
        for x in out.iter_mut() {
            *x = x.wrapping_abs();
        }
        out
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn bitmask_over_64_elements(input: [i8; 64]) -> [i8; 64] {
    let mut out = input;
    for x in out.iter_mut() {
        *x = x.wrapping_abs();
    }
    out
}

pub fn trigger_bitmask_over_64_elements() {
    let _ = bitmask_over_64_elements([0i8; 64]);
}
