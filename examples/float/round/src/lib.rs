/// IEEE 754 round_half_to_even (banker's rounding) observable via f64::round_ties_even
/// and arithmetic results; also covers floor / ceil / trunc / fract intrinsics.
/// Verifiers that approximate rounding as "round half up" will misclassify the tie cases.
pub fn float_round() {
    // round(): round half away from zero (Rust default)
    let _r_half_pos: f64 = (0.5_f64).round();  // 1.0
    let _r_half_neg: f64 = (-0.5_f64).round(); // -1.0
    let _r_1_5: f64 = (1.5_f64).round();       // 2.0
    let _r_2_5: f64 = (2.5_f64).round();       // 3.0 (away from zero, not banker's)

    // floor / ceil / trunc / fract
    let _floor: f64 = (2.7_f64).floor();  // 2.0
    let _ceil: f64 = (2.3_f64).ceil();    // 3.0
    let _trunc: f64 = (-2.7_f64).trunc(); // -2.0 (toward zero)
    let _fract: f64 = (2.7_f64).fract();  // 0.7 (approximately)

    // round_ties_even (IEEE 754 default mode, stable since 1.77)
    let _rte_0_5: f64 = (0.5_f64).round_ties_even(); // 0.0 (rounds to even)
    let _rte_1_5: f64 = (1.5_f64).round_ties_even(); // 2.0 (rounds to even)
    let _rte_2_5: f64 = (2.5_f64).round_ties_even(); // 2.0 (rounds to even)
    let _rte_3_5: f64 = (3.5_f64).round_ties_even(); // 4.0 (rounds to even)
}
