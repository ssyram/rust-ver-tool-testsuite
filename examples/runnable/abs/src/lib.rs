/// i32 absolute value via if-else. Hand-rolled to avoid std::abs (which is
/// builtin in some toolchains and may not be in hax-lean prelude).
pub fn my_abs(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}
