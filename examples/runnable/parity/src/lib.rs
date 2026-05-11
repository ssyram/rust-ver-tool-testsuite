/// Parity check on i32 → bool. Negative input mod 2 in Rust may be 0 / -1, so
/// 比对 == 0 to normalize.
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}
