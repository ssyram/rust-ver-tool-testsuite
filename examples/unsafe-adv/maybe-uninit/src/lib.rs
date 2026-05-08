/// `MaybeUninit` write + `assume_init`.
pub fn unsafe_maybe_uninit() {
    use std::mem::MaybeUninit;
    let mut buf: MaybeUninit<i32> = MaybeUninit::uninit();
    buf.write(42);
    let v = unsafe { buf.assume_init() };
    let _ = v;
}
