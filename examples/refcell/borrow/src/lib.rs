use std::cell::RefCell;

/// Mutate through RefCell using runtime borrow checks. No conflicting borrows.
/// Exercises interior mutability, borrow_mut/borrow runtime tracking.
pub fn refcell_borrow_mut() {
    let c = RefCell::new(0i32);
    *c.borrow_mut() = 42;
    let v = *c.borrow();
    let _ = v;
}
