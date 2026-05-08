/// Raw pointer write via `ptr::write`.
pub fn unsafe_ptr_write() {
    let mut x: i32 = 0;
    let p: *mut i32 = &mut x;
    unsafe { std::ptr::write(p, 99); }
    let _ = x;
}
