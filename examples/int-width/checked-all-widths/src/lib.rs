/// checked_* arithmetic across i8, i16, i32, i64, i128: returns None on overflow.
/// Tests whether a verifier correctly propagates the Option discriminant for
/// each distinct integer width.  i128 checked_add is the hardest case; a verifier
/// whose SMT backend lacks native 128-bit support may spuriously return Some.
pub fn int_width_checked_all_widths() {
    // i8
    let _a: Option<i8>   = i8::MAX.checked_add(1_i8);   // None
    let _b: Option<i8>   = i8::MAX.checked_add(0_i8);   // Some(127)

    // i16
    let _c: Option<i16>  = i16::MAX.checked_add(1_i16); // None
    let _d: Option<i16>  = i16::MIN.checked_sub(1_i16); // None

    // i32
    let _e: Option<i32>  = i32::MAX.checked_mul(2_i32); // None
    let _f: Option<i32>  = i32::MIN.checked_div(-1_i32); // None (overflow)

    // i64
    let _g: Option<i64>  = i64::MAX.checked_add(1_i64); // None
    let _h: Option<i64>  = (i64::MAX / 2).checked_mul(3_i64); // None

    // i128 — primary stress: verifier must model 128-bit Option<i128>
    let _i: Option<i128> = i128::MAX.checked_add(1_i128); // None
    let _j: Option<i128> = (i128::MAX / 2_i128).checked_mul(3_i128); // None
    let _k: Option<i128> = i128::MIN.checked_div(-1_i128); // None (overflow)
    let _l: Option<i128> = 1_i128.checked_add(1_i128);    // Some(2)
}
