/// `as` cast sign-extension (signed narrower -> wider) and zero-extension
/// (unsigned narrower -> wider).  These are distinct operations at the bit level;
/// verifiers must choose the correct extension for the source type.
pub fn int_width_cast_sign_extend() {
    // Sign-extension: i8 -> i32 (negative value fills high bits with 1)
    let neg8: i8 = -1_i8;            // 0xFF in bits
    let _s1: i32 = neg8 as i32;      // -1 (0xFFFF_FFFF), not 255

    let neg8b: i8 = i8::MIN;         // -128 = 0x80
    let _s2: i32 = neg8b as i32;     // -128 (0xFFFF_FF80)

    // Sign-extension: i16 -> i64
    let neg16: i16 = -1000_i16;
    let _s3: i64 = neg16 as i64;     // -1000

    // Zero-extension: u8 -> u32 (high bits filled with 0)
    let pos8: u8 = 255_u8;           // 0xFF
    let _z1: u32 = pos8 as u32;      // 255 (0x0000_00FF)

    // Zero-extension: u16 -> u64
    let pos16: u16 = u16::MAX;       // 0xFFFF
    let _z2: u64 = pos16 as u64;     // 65535

    // u8 -> i32: zero-extend first, then reinterpret (same as u32 -> i32 without sign issues)
    let _z3: i32 = 200_u8 as i32;   // 200 (not -56)

    // i8 -> u32: sign-extend to i32 then reinterpret as u32
    let _sz: u32 = (-1_i8) as u32;  // 0xFFFF_FFFF = 4294967295
}
