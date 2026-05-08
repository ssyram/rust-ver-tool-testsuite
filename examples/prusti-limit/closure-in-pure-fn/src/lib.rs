// Prusti limitation: closures inside `#[pure]` functions are not supported.
//
// Even a trivially pure closure (one that has no side effects and always
// terminates) inside a `#[pure]`-annotated function causes two errors:
//
//   1. [Prusti: invalid specification] use of impure function "std::ops::Fn::call"
//      in pure code is not allowed
//   2. [Prusti: unsupported feature] unsupported constant type ... Closure(...)
//
// Since `#[pure]` functions are used in specifications, this limits the kinds
// of helper functions that can be reused inside contracts.
//
// Source: https://github.com/viperproject/prusti-dev/issues/1543

/// Check whether `n` is positive using an immediately-invoked closure.
/// The closure is trivially pure, but Prusti rejects `#[pure]` functions
/// that contain any closure literal or closure call.
pub fn is_positive(n: i32) -> bool {
    (|| n > 0)()
}

pub fn trigger_closure_in_pure_fn() {
    let _ = is_positive(5);
    let _ = is_positive(-3);
}
