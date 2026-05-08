/// Higher-Ranked Trait Bound: `for<'a> Fn(&'a i32) -> i32`.
pub fn hrtb_apply() {
    fn apply<F>(f: F) -> i32 where F: for<'a> Fn(&'a i32) -> i32 {
        let x = 10;
        f(&x)
    }
    let _ = apply(|r| *r + 1);
}
