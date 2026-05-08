/// Returning a closure as `impl Fn(_) -> _`.
pub fn return_impl_fn() {
    fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
        move |y| x + y
    }
    let f = make_adder(7);
    let _ = f(5);
    let _ = f(100);
}
