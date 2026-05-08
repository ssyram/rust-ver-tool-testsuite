/// `T: 'static` bound and `Box<dyn Any>` upcasting.
pub fn static_bound() {
    fn boxed<T: 'static>(x: T) -> Box<dyn std::any::Any> {
        Box::new(x)
    }
    let b1 = boxed(42i32);
    let b2 = boxed(true);
    let _ = (b1.is::<i32>(), b2.is::<bool>());
}
