/// Multiple lifetimes with outlives constraint `'b: 'a`.
pub fn multi_lifetime() {
    fn pick<'a, 'b: 'a>(x: &'a i32, y: &'b i32, take_y: bool) -> &'a i32 {
        if take_y { y } else { x }
    }
    let a = 1;
    let b = 2;
    let _ = pick(&a, &b, true);
    let _ = pick(&a, &b, false);
}
