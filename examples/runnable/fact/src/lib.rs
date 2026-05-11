/// Self-recursive factorial on i32. Input must be small enough not to overflow.
pub fn fact(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * fact(n - 1)
    }
}
