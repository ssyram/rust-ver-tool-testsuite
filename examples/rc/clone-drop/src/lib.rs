use std::rc::Rc;

/// Allocate Rc, clone (share ownership), deref through both, drop.
/// Exercises Rc allocation, ref-count management, Drop ordering.
pub fn rc_clone_drop() {
    let a = Rc::new(42i32);
    let b = Rc::clone(&a);
    let _ = *a + *b;
    drop(a);
    drop(b);
}
