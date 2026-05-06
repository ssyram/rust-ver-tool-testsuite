/// Read through a raw const pointer (well-defined: pointer to live local).
/// Exercises *const T, unsafe block, raw deref.
pub fn raw_ptr_read() {
    let x: i32 = 42;
    let p: *const i32 = &x;
    let v = unsafe { *p };
    let _ = v;
}
