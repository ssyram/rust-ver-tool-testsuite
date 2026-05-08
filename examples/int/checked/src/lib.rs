/// `checked_*` arithmetic returning `Option<T>` on overflow.
pub fn int_checked() {
    let a: i32 = i32::MAX - 1;
    let _: Option<i32> = a.checked_add(2);
    let _: Option<i32> = a.checked_add(0);
    let _: Option<i32> = (i32::MIN).checked_sub(1);
}
