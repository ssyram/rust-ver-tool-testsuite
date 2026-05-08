// Aeneas limitation: incorrect code generation for higher-order functions that
// accept an `FnMut` closure returning the unit type `()`.
//
// `FnMut::call_mut` returns `(output, updated_closure)`.  When the output is
// `()`, Aeneas generates the binding incorrectly:
//   `let p1 <- call_mut p 0`    -- wrong: discards updated closure
// instead of the correct:
//   `let (_, p1) <- call_mut p 0`
//
// The generated Lean code fails to type-check because the tuple is not
// destructured.
//
// Source: https://github.com/AeneasVerif/aeneas/issues/960

pub fn apply_twice<P: FnMut(usize)>(mut p: P) {
    p(0);
    p(1);
}

pub fn apply_fn_mut_unit<F: FnMut(u32)>(mut f: F, x: u32) {
    f(x);
    f(x);
}

pub fn trigger_fnmut_unit_return() {
    let mut count = 0usize;
    apply_twice(|_| { count += 1; });
    apply_fn_mut_unit(|_| { count += 1; }, 7);
}
