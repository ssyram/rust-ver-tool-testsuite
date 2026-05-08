// Prusti limitation: closures are not supported.
//
// Prusti reports "[Prusti: unsupported feature] this is unsupported, because it
// uses closures" for any function that defines or calls a closure. The feature
// is tracked in prusti-dev issue #169 and PR #138; the user guide section on
// closures is explicitly marked "NOT YET SUPPORTED".
//
// Source: https://viperproject.github.io/prusti-dev/user-guide/verify/closure.html
//         https://github.com/viperproject/prusti-dev/issues/169

/// Apply a caller-supplied increment to `x` via a closure.
pub fn apply_increment(x: u32) -> u32 {
    let inc = |v: u32| v + 1;
    inc(x)
}

pub fn trigger_closures_unsupported() {
    let _ = apply_increment(10);
}
