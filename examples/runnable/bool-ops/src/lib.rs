/// Boolean operator composite.
pub fn and_or_not(a: bool, b: bool, c: bool) -> bool {
    (a && b) || !c
}
