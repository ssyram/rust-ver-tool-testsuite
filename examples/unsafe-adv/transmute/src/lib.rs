/// `mem::transmute` between same-size types.
pub fn unsafe_transmute() {
    let x: u32 = 0x12345678;
    let bytes: [u8; 4] = unsafe { std::mem::transmute(x) };
    let _ = bytes;
}
