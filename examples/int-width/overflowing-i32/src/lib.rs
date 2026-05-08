/// i32 overflowing_* arithmetic: returns (wrapped_value, did_overflow: bool).
/// Tests whether verifiers can track the boolean overflow flag alongside the
/// integer result.  The tuple return type forces the verifier to model a
/// compound value, which is harder than a simple Option<T>.
pub fn int_width_overflowing_i32() {
    let max: i32 = i32::MAX;
    let min: i32 = i32::MIN;

    // Overflow cases: expect (wrapped, true)
    let (v1, f1) = max.overflowing_add(1_i32);   // (-2^31, true)
    let _ = (v1, f1);

    let (v2, f2) = min.overflowing_sub(1_i32);   // (max, true)
    let _ = (v2, f2);

    let (v3, f3) = max.overflowing_mul(2_i32);   // (-2, true)
    let _ = (v3, f3);

    // MIN / -1 overflow
    let (v4, f4) = min.overflowing_div(-1_i32);  // (min, true)
    let _ = (v4, f4);

    // Non-overflow cases: expect (result, false)
    let (v5, f5) = 100_i32.overflowing_add(200_i32); // (300, false)
    let _ = (v5, f5);
}
