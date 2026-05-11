/// Integer power via self-recursion (no loop, friendly to翻译类工具).
pub fn pow_n(base: i32, exp: u32) -> i32 {
    if exp == 0 {
        1
    } else {
        base * pow_n(base, exp - 1)
    }
}
