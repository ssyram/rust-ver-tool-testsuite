/// Digit sum on u32 via recursion (mod / div).
pub fn digit_sum(n: u32) -> u32 {
    if n == 0 {
        0
    } else {
        n % 10 + digit_sum(n / 10)
    }
}
