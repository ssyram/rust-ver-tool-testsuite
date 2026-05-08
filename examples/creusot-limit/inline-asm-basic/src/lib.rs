// Creusot limitation: inline assembly is not supported.
// The MIR terminator `InlineAsm { .. }` is listed alongside `CoroutineDrop`
// and `Yield` as `unreachable!` in Creusot's terminator translation, meaning
// any function containing `asm!` will panic the compiler plugin.
//
// Source: creusot/src/translation/function/terminator.rs – InlineAsm branch
//         marked unreachable!("{:?}", terminator.kind)
//
// cargo check: passes on x86_64 (asm! is stable since Rust 1.59)
// Creusot: panics/crashes — InlineAsm terminator hits unreachable!()

#[cfg(target_arch = "x86_64")]
pub fn nop_via_asm() {
    unsafe {
        std::arch::asm!("nop");
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn nop_via_asm() {
    // no-op on non-x86 to keep cargo check passing everywhere
}
