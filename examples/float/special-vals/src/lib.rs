/// IEEE 754 special values: NaN, +inf, -inf, -0.0, and the smallest subnormal f64;
/// verifier heavy-hitters — many tools silently mishandle NaN propagation or conflate ±0.
pub fn float_special_vals() {
    // NaN
    let nan: f64 = f64::NAN;
    let _nan_add: f64 = nan + 1.0;      // still NaN
    let _nan_mul: f64 = nan * 0.0;      // still NaN (not 0)

    // infinities
    let inf: f64 = f64::INFINITY;
    let neg_inf: f64 = f64::NEG_INFINITY;
    let _inf_add: f64 = inf + 1.0;      // +inf
    let _inf_sub: f64 = inf + neg_inf;  // NaN
    let _inf_mul: f64 = inf * 0.0;      // NaN

    // negative zero: distinct bit pattern, but == 0.0
    let neg_zero: f64 = -0.0_f64;
    let pos_zero: f64 = 0.0_f64;
    let _eq_zero: bool = neg_zero == pos_zero;  // true in IEEE 754
    let _div_neg_zero: f64 = 1.0_f64 / neg_zero; // -inf
    let _div_pos_zero: f64 = 1.0_f64 / pos_zero; // +inf

    // smallest positive subnormal f64
    let subnormal: f64 = f64::MIN_POSITIVE * f64::EPSILON; // well below MIN_POSITIVE
    let _sub_add: f64 = subnormal + subnormal;
}
