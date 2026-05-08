/// FnOnce closure that consumes captured by-value state.
pub fn closure_fn_once() {
    fn run_once<F: FnOnce() -> i32>(f: F) -> i32 { f() }
    let s = String::from("owned");
    let _ = run_once(move || {
        let _drop = s;
        42
    });
}
