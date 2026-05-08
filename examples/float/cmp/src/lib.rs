/// Float comparison: == (reflexivity violation for NaN), partial_cmp returning None for NaN,
/// and ordering of infinities; the NaN != NaN axiom is the single most common verifier pitfall.
pub fn float_cmp() {
    use std::cmp::Ordering;

    let nan: f64 = f64::NAN;

    // NaN is not equal to itself — violates reflexivity
    let _nan_ne_self: bool = nan != nan; // true

    // partial_cmp returns None for NaN comparisons
    let _nan_cmp: Option<Ordering> = nan.partial_cmp(&nan);       // None
    let _nan_cmp2: Option<Ordering> = nan.partial_cmp(&1.0_f64);  // None

    // normal ordering
    let a: f64 = 1.0_f64;
    let b: f64 = 2.0_f64;
    let _lt: Option<Ordering> = a.partial_cmp(&b); // Some(Less)
    let _eq: Option<Ordering> = a.partial_cmp(&a); // Some(Equal)

    // infinities compare correctly
    let _inf_gt: Option<Ordering> = f64::INFINITY.partial_cmp(&f64::MAX); // Some(Greater)
    let _neginf_lt: Option<Ordering> = f64::NEG_INFINITY.partial_cmp(&f64::MIN); // Some(Less)

    // -0.0 == +0.0
    let _zeros_eq: bool = (-0.0_f64) == 0.0_f64; // true
}
