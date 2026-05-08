/// Cyclic well-formedness in trait clauses: `Trait<T>` requires
/// `T::Unwrapped: Copy`, and proving `T::Val: Copy` reaches back into
/// the same clause. Charon's known-failure (incorrect-cyclic-wf).
pub fn cyclic_bound_use() {
    pub struct Wrap<T: Copy>(T);

    pub trait Unwrap {
        type Unwrapped;
    }
    impl<T: Copy> Unwrap for Wrap<T> {
        type Unwrapped = T;
    }

    pub trait Trait<T: Unwrap>
    where
        T::Unwrapped: Copy,
    {
    }

    pub trait HasVal {
        type Val: Copy;
    }

    fn f<T: HasVal, A: Trait<Wrap<T::Val>>>() {}

    struct Anchor;
    impl HasVal for Anchor {
        type Val = i32;
    }
    struct Witness;
    impl Trait<Wrap<i32>> for Witness {}

    f::<Anchor, Witness>();
}
