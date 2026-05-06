use std::sync::Arc;

/// Allocate Arc, clone (atomic ref-count), deref, drop.
/// Exercises Arc allocation, atomic operations, Send/Sync.
pub fn arc_clone_drop() {
    let a = Arc::new(42i32);
    let b = Arc::clone(&a);
    let _ = *a + *b;
}
