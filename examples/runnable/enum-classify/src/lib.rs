/// Custom enum + match. Entry fn takes i32 (v1 type matrix), internally maps
/// to Sign and matches on it → i32 return.
pub enum Sign {
    Neg,
    Zero,
    Pos,
}

fn sign_of(n: i32) -> Sign {
    if n < 0 {
        Sign::Neg
    } else if n == 0 {
        Sign::Zero
    } else {
        Sign::Pos
    }
}

pub fn classify_sign(n: i32) -> i32 {
    match sign_of(n) {
        Sign::Neg => -1,
        Sign::Zero => 0,
        Sign::Pos => 1,
    }
}
