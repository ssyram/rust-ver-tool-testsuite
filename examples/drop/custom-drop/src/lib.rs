/// Custom `impl Drop` — destructors run in reverse declaration order.
pub fn custom_drop_order() {
    use std::cell::Cell;
    struct Loud<'a> { id: u32, log: &'a Cell<u32> }
    impl<'a> Drop for Loud<'a> {
        fn drop(&mut self) {
            self.log.set(self.log.get() * 10 + self.id);
        }
    }
    let log = Cell::new(0);
    {
        let _a = Loud { id: 1, log: &log };
        let _b = Loud { id: 2, log: &log };
    }
    let _ = log.get();
}
