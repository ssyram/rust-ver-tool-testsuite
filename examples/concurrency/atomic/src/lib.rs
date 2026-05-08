/// `AtomicUsize` with explicit `Ordering`.
pub fn atomic_seqcst() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let a = AtomicUsize::new(0);
    a.fetch_add(1, Ordering::SeqCst);
    a.fetch_add(2, Ordering::Relaxed);
    let _ = a.load(Ordering::SeqCst);
    a.store(99, Ordering::Release);
    let _ = a.load(Ordering::Acquire);
}
