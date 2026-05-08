/// `wrapping_*` arithmetic intrinsics on u8 (forces overflow modulo 2^N).
pub fn int_wrapping() {
    let a: u8 = 250;
    let b: u8 = 20;
    let _ = a.wrapping_add(b);
    let _ = a.wrapping_mul(b);
    let _ = a.wrapping_sub(b.wrapping_mul(13));
}
