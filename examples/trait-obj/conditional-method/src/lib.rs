/// Trait method gated by `where Self: Clone` — the method is only
/// available on impls that are `Clone`. Charon's known-failure;
/// trait-object resolution under conditional methods stresses verifiers.
pub fn conditional_method() {
    struct NeedsClone<T: Clone>(T);

    trait Trait {
        fn method(&self) -> NeedsClone<Self>
        where
            Self: Clone + Sized,
        {
            NeedsClone(self.clone())
        }
    }

    #[derive(Clone)]
    struct Foo(i32);
    impl Trait for Foo {}

    let f = Foo(7);
    let _ = f.method();
}
