/// f64::total_cmp (stable since 1.62): imposes a total order on all f64 values including NaN;
/// NaN is placed above +inf in this order, and -NaN below -inf.
/// This distinguishes verifiers that model only partial order from those aware of total_cmp.
pub fn float_total_order() {
    use std::cmp::Ordering;

    let nan: f64 = f64::NAN;
    let inf: f64 = f64::INFINITY;
    let neg_inf: f64 = f64::NEG_INFINITY;

    // NaN > +inf in total order
    let _nan_gt_inf: Ordering = nan.total_cmp(&inf);          // Greater
    // -NaN < -inf in total order
    let _neg_nan_lt_neg_inf: Ordering = (-nan).total_cmp(&neg_inf); // Less

    // NaN == NaN under total_cmp (reflexive, unlike ==)
    let _nan_eq_nan: Ordering = nan.total_cmp(&nan);          // Equal

    // normal ordering preserved
    let _one_lt_two: Ordering = (1.0_f64).total_cmp(&2.0_f64); // Less

    // -0.0 < +0.0 in total order (unlike ==)
    let _neg_zero_lt_pos_zero: Ordering = (-0.0_f64).total_cmp(&0.0_f64); // Less
}
