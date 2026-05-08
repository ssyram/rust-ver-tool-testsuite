/// `thread::spawn` + `Arc<Mutex<_>>` + `join`.
pub fn thread_mutex_join() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    let counter = Arc::new(Mutex::new(0i32));
    let mut hs = vec![];
    for _ in 0..2 {
        let c = Arc::clone(&counter);
        hs.push(thread::spawn(move || {
            let mut g = c.lock().unwrap();
            *g += 1;
        }));
    }
    for h in hs { h.join().unwrap(); }
    let _ = *counter.lock().unwrap();
}
