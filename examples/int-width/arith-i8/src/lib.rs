/// i8 (8-bit signed) basic arithmetic: +, -, *, /, % on boundary values.
/// Tests whether verifiers correctly model 8-bit two's-complement range
/// [-128, 127] and detect overflow at the type's natural boundary.
pub fn int_width_arith_i8() {
    let min: i8 = i8::MIN; // -128
    let max: i8 = i8::MAX; // 127
    let zero: i8 = 0_i8;
    let neg_one: i8 = -1_i8;

    // Non-overflowing arithmetic
    let _add: i8 = max.wrapping_add(neg_one);  // 126
    let _sub: i8 = min.wrapping_sub(neg_one);  // -127
    let _mul: i8 = 10_i8 * 10_i8;             // 100, fits in i8
    let _div: i8 = max / 2_i8;                 // 63
    let _rem: i8 = max % 3_i8;                 // 1

    // Boundary: MIN / -1 is UB in C but wrapping in Rust's wrapping_div
    let _divw: i8 = min.wrapping_div(neg_one); // wraps to -128

    // Zero denominators kept absent — division by zero is a separate concern
    let _ = zero + zero; // trivial identity
}
