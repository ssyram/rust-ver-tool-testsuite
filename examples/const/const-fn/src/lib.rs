/// `const fn` evaluated at compile time, used as array length.
pub fn const_fn_eval() {
    const fn sq(x: i32) -> i32 { x * x }
    const SIZE: usize = sq(4) as usize;
    let arr: [i32; SIZE] = [0; SIZE];
    let _ = arr.len();
}
