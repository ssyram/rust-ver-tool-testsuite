/// i128 (128-bit signed) basic arithmetic: +, -, *, /, wrapping variants.
/// Primary differentiator: SMT solvers backed by fixed-width bitvector theories
/// (e.g. CBMC's internal solver, some Z3 configurations) may silently mis-model
/// 128-bit integers or time out on multiplication, because QF_BV128 is uncommon.
pub fn int_width_arith_i128() {
    let min: i128 = i128::MIN; // -2^127
    let max: i128 = i128::MAX; // 2^127 - 1
    let big: i128 = 0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_fffei128; // max - 1

    // Non-overflowing: verifier must track 128-bit value
    let _add: i128 = big + 1_i128;     // == max
    let _sub: i128 = min + 1_i128;     // -2^127 + 1

    // wrapping on exact boundary
    let _wrap_add: i128 = max.wrapping_add(1_i128); // wraps to min
    let _wrap_sub: i128 = min.wrapping_sub(1_i128); // wraps to max

    // Multiplication staying inside range
    let _mul: i128 = 1_000_000_000_i128 * 1_000_000_000_i128; // 10^18, < 2^63

    // Wider multiplication: result requires full 128-bit precision
    let a: i128 = i64::MAX as i128;             // 2^63 - 1
    let b: i128 = i64::MAX as i128;
    let _wide: i128 = a * b;                     // ~2^126, still fits in i128

    // MIN / -1 via wrapping_div (standard overflow)
    let _divw: i128 = min.wrapping_div(-1_i128); // wraps to min
}
