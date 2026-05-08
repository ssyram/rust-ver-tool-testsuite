/// `as` cast truncation: wider integer to narrower integer.
/// The `as` operator in Rust truncates the value to the target width (keeps
/// low bits).  Verifiers must model bit-level truncation, not arithmetic
/// range reduction — the two diverge for values outside the target range.
pub fn int_width_cast_truncate() {
    // u32 -> u8: 0x1_FF = 511 -> 255 (low 8 bits)
    let a: u32 = 0x1_FFu32;
    let _t1: u8 = a as u8;  // 255

    // u32 -> u8: 256 -> 0
    let _t2: u8 = 256_u32 as u8; // 0

    // i32 -> i8: value outside [-128, 127]
    let _t3: i8 = 200_i32 as i8;  // -56 (200 - 256)
    let _t4: i8 = 384_i32 as i8;  // 128 - 256 = -128... actually 384 & 0xFF = 128 as i8 = -128
    let _t5: i8 = (-130_i32) as i8; // 126 (two's complement wrap)

    // u64 -> u32: keep low 32 bits
    let b: u64 = 0x1_0000_0001u64;
    let _t6: u32 = b as u32; // 1

    // i64 -> i16: keep low 16 bits, reinterpret as i16
    let c: i64 = 0x0001_FFFF_i64; // 131071
    let _t7: i16 = c as i16; // -1 (0xFFFF interpreted as i16)
}
