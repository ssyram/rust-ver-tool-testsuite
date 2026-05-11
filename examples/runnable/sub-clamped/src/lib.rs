/// Subtraction with hand-rolled boundary check: clamp result to >= 0.
/// Exercises if-branch on i32 comparison without builtin checked_*.
pub fn sub_clamped(a: i32, b: i32) -> i32 {
    if a < b {
        0
    } else {
        a - b
    }
}
