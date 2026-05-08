// Prusti limitation: for-loops over iterators are not supported.
//
// Rust's `for x in iter` syntax desugars to calls to `Iterator::next`, which
// involves magic wands (borrow expiry) that Prusti cannot encode. The test
// `tests/verify/pass/loop-invs/for_iter.rs` and
// `tests/verify/pass/loop-invs/simple_iterator.rs` are both listed as
// `ignore-test` in the tracking issue because "magic wands in loop invariants"
// are not yet supported.
//
// This means that any function using `for x in collection` or a method chain
// like `.iter().map(...).filter(...)` cannot be verified by Prusti today.
//
// Source: https://github.com/viperproject/prusti-dev/issues/543
//         (entries: "magic wands in loop invariants" / for_iter.rs / simple_iterator.rs)

/// Sum all elements in a slice using a for-loop (desugars to Iterator::next).
pub fn sum_slice(values: &[i32]) -> i32 {
    let mut total = 0i32;
    for &v in values {
        total += v;
    }
    total
}

pub fn trigger_for_loop_iterator() {
    let data = [1i32, 2, 3, 4, 5];
    let _ = sum_slice(&data);
}
