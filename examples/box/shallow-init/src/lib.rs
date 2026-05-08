/// `vec![<expr-with-early-return>]` lowers to `ShallowInitBox` MIR
/// before the box is fully initialised. Charon's known-failure
/// (issue-393); other verifiers may or may not handle the half-init box.
pub fn shallow_init_box() {
    fn next(b: bool) -> Option<Vec<u8>> {
        let v = vec![if b { 42 } else { return None }];
        Some(v)
    }
    let _ = next(true);
    let _ = next(false);
}
