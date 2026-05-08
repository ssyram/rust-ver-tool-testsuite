/// `Box<dyn Fn(_) -> _>` — closure as trait object via vtable.
pub fn boxed_dyn_fn() {
    let fs: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(|x| x + 1),
        Box::new(|x| x * 2),
        Box::new(|x| x - 3),
    ];
    let mut acc = 10;
    for f in &fs {
        acc = f(acc);
    }
    let _ = acc;
}
