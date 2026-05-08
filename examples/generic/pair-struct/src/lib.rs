/// Generic struct with two type parameters and an inherent method.
pub fn generic_pair() {
    struct Pair<A, B> { a: A, b: B }
    impl<A, B> Pair<A, B> {
        fn new(a: A, b: B) -> Self { Pair { a, b } }
        fn into_tuple(self) -> (A, B) { (self.a, self.b) }
    }
    let p = Pair::new(1u32, true);
    let _ = p.into_tuple();
}
