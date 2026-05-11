/// Hand-rolled saturating u8 addition (no builtin saturating_add).
/// 255 is u8::MAX, encoded literally to avoid relying on intrinsics.
pub fn sat_add_u8(a: u8, b: u8) -> u8 {
    let max: u8 = 255;
    let room: u8 = max - a;
    if b > room {
        max
    } else {
        a + b
    }
}
