/// Implicit panic edge from `a / b`. Runtime exercises only safe inputs.
pub fn div_zero_path() {
    fn divide(a: i32, b: i32) -> i32 { a / b }
    let _ = divide(10, 2);
    let _ = divide(7, 1);
}
