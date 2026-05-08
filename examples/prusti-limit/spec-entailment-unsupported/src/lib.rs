// Prusti limitation: specification entailments (higher-order function contracts)
// are not supported.
//
// Prusti's user guide describes a `f |= |args| [requires(...), ensures(...)]`
// syntax for constraining the contract of a closure/function-pointer argument,
// but the entire feature is marked "NOT YET SUPPORTED". There is no way to
// attach a verifiable pre/post-condition to a generic Fn parameter today.
//
// This entry exercises the plain-Rust pattern: a generic higher-order function
// that accepts a callback with a HRTB-style `for<'a>` bound.
//
// Source: https://viperproject.github.io/prusti-dev/user-guide/verify/spec_ent.html

/// Return the result of applying `f` to `value`.
/// There is no way to specify the required contract of `f` in Prusti.
pub fn apply<F: Fn(i32) -> i32>(f: F, value: i32) -> i32 {
    f(value)
}

/// Same idea but with an explicit higher-ranked lifetime bound.
pub fn apply_ref<F>(f: F, value: &i32) -> i32
where
    F: for<'a> Fn(&'a i32) -> i32,
{
    f(value)
}

pub fn trigger_spec_entailment_unsupported() {
    let _ = apply(|x| x + 1, 10);
    let val = 42i32;
    let _ = apply_ref(|x| *x * 2, &val);
}
