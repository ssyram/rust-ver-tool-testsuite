// Creusot limitation: iterating over a generic `I: Iterator` with a `for` loop
// fails because Creusot requires the iterator type to implement `creusot_std::Iterator`
// (Creusot's own iterator trait with a `produces` specification).  A plain
// `std::iter::Iterator` bound carries no such spec, so Creusot emits:
//
//   error[E0277]: the trait bound `&mut I: creusot_contracts::IntoIterator` is not satisfied
//
// Source: Issue #1285 "Creusot rejects these generic definitions with trait constraints"
//         (minimal example `pub fn f<I: Iterator>(i: &mut I) { for _ in i {} }`)
//
// cargo check: passes (standard Rust generic iterator loop)
// Creusot: compile-time error — missing `creusot_std::Iterator` impl for &mut I

pub fn drain_generic_iter<I: Iterator>(iter: I) -> usize {
    let mut count = 0usize;
    for _ in iter {
        count += 1;
    }
    count
}

pub fn trigger_drain_generic_iter() {
    let v: Vec<i32> = vec![1, 2, 3];
    let _ = drain_generic_iter(v.into_iter());
}
