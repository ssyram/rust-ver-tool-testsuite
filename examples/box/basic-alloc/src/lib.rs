/// Allocate a boxed int, deref it, drop. Exercises Box heap allocation, deref, Drop.
pub fn alloc_deref_drop() {
    let b: Box<i32> = Box::new(42);
    let _ = *b;
}
