/// u128 (128-bit unsigned) basic arithmetic: +, -, *, /, %, wrapping variants.
/// Like i128 but unsigned; tests whether verifiers handle the full [0, 2^128-1]
/// domain.  u128 multiplication is especially demanding for SMT backends because
/// the result can use all 128 bits without sign-extension shortcuts.
pub fn int_width_arith_u128() {
    let max: u128 = u128::MAX; // 2^128 - 1
    let big: u128 = u128::MAX - 1;

    // Non-overflowing add
    let _add: u128 = big + 1_u128; // == max

    // wrapping overflow at max
    let _wrap_add: u128 = max.wrapping_add(1_u128); // 0
    let _wrap_sub: u128 = 0_u128.wrapping_sub(1_u128); // max

    // Multiplication: product of two 64-bit-range values
    let a: u128 = u64::MAX as u128;    // 2^64 - 1
    let b: u128 = 2_u128;
    let _mul: u128 = a * b;            // 2^65 - 2, > u64::MAX, requires u128

    // Division and remainder (no overflow possible for unsigned div)
    let _div: u128 = max / 3_u128;
    let _rem: u128 = max % 3_u128;    // 0 (since 2^128-1 is divisible by 3)

    // Checked: should return None on overflow, Some otherwise
    let _chk: Option<u128> = max.checked_add(1_u128); // None
    let _chk2: Option<u128> = big.checked_add(1_u128); // Some(max)
}
