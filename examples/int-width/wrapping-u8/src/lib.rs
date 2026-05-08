/// u8 wrapping_* arithmetic: modulo 256 semantics explicitly requested.
/// Classic verifier differentiator: Kani / CBMC model wrapping_add as modular
/// addition and can reason about the result; tools that treat all integers as
/// mathematical integers (no overflow model) will mis-model this.
pub fn int_width_wrapping_u8() {
    let a: u8 = 255_u8; // u8::MAX
    let b: u8 = 1_u8;

    let _add: u8 = a.wrapping_add(b);      // 0
    let _sub: u8 = 0_u8.wrapping_sub(b);  // 255
    let _mul: u8 = 200_u8.wrapping_mul(2_u8); // 144 (400 mod 256)

    // Composition: wrapping_add followed by wrapping_mul
    let c: u8 = 250_u8;
    let d: u8 = c.wrapping_add(10_u8);     // 4
    let _e: u8 = d.wrapping_mul(64_u8);    // 256 mod 256 = 0

    // Chain: wrapping_sub producing a value mid-range
    let _f: u8 = 10_u8.wrapping_sub(20_u8); // 246
}
