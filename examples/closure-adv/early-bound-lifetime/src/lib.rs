/// Closure → `fn(&'a T)` coercion forces the closure's reference arg
/// into an early-bound lifetime relative to the fn pointer signature.
/// Charon's known-failure (issue-1010) on the early-bound region elaboration.
pub fn early_bound_closure_arg() {
    fn make_early<'a, T>(_: fn(&'a T)) {}

    let _ = |_: &u8| ();
    make_early(|_: &u16| ());
}
