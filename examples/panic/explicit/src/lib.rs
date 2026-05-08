/// Explicit `panic!` on a branch that won't be taken at runtime.
/// Verifier should still see the panic edge in CFG.
pub fn explicit_panic() {
    fn maybe_panic(x: i32) -> i32 {
        if x < 0 { panic!("negative input"); }
        x * 2
    }
    let _ = maybe_panic(5);
    let _ = maybe_panic(0);
}
