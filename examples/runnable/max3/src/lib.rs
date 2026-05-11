/// Max of three i32 via nested if-else.
pub fn max3(a: i32, b: i32, c: i32) -> i32 {
    let m = if a > b { a } else { b };
    if m > c {
        m
    } else {
        c
    }
}
