/// Bit-shift operations with overflow boundary: shift amount >= type width panics
/// in debug mode (UB in C).  Uses wrapping_shl/wrapping_shr for safe variants.
/// Every verifier should detect the panic boundary but they differ in whether
/// they report it as UB, assertion violation, or unreachable.
pub fn int_width_shift_overflow() {
    // Safe shifts (amount < width)
    let _s1: u8  = 1_u8  << 7;   // 128, legal (shift by 7 < 8)
    let _s2: u16 = 1_u16 << 15;  // 32768
    let _s3: u32 = 1_u32 << 31;  // 2^31
    let _s4: u64 = 1_u64 << 63;  // 2^63
    let _s5: u8  = 128_u8 >> 7;  // 1

    // wrapping_shl: shift amount taken modulo width (no panic)
    let _w1: u8  = 1_u8.wrapping_shl(8);  // == 1_u8.wrapping_shl(0) = 1
    let _w2: u8  = 1_u8.wrapping_shl(9);  // == 1_u8.wrapping_shl(1) = 2
    let _w3: u32 = 1_u32.wrapping_shl(32); // 1 (32 mod 32 = 0)
    let _w4: u32 = 1_u32.wrapping_shl(33); // 2

    // wrapping_shr on i32 (arithmetic right shift)
    let _w5: i32 = (-1_i32).wrapping_shr(1);  // -1 (sign bit propagates)
    let _w6: i32 = (-1_i32).wrapping_shr(32); // -1 (wraps to shift by 0)

    // checked_shl: returns None when shift >= width
    let _c1: Option<u8>  = 1_u8.checked_shl(7);  // Some(128)
    let _c2: Option<u8>  = 1_u8.checked_shl(8);  // None (shift == width)
    let _c3: Option<u32> = 1_u32.checked_shl(31); // Some(2^31)
    let _c4: Option<u32> = 1_u32.checked_shl(32); // None
}
