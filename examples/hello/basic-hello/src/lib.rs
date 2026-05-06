/// Smoke entry: zero-arg pub fn whose body exercises trivial Rust syntax.
/// Used as the simplest possible target for end-to-end runner verification.
pub fn hello() {
    let _ = 1 + 1;
}
