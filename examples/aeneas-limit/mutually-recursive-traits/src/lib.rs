// Aeneas limitation: mutually-recursive traits with associated types cannot be
// handled.  When Trait1 has an associated type T: Trait2 and Trait2 supertrait
// Trait1, Aeneas emits:
//
//   "[Error] Found an associated type in a trait declaration; trait associated
//    types are usually lifted to become parameters of the trait definition, but
//    this can fail with mutually-recursive traits as well as GATs.  Aeneas
//    cannot handle such types today, and the generated code will likely be
//    incorrect."
//
// This is an Aeneas-level limitation (not just Charon): Charon successfully
// produces LLBC, but Aeneas's `SymbolicToPure` pass cannot lift the associated
// type parameter in the presence of the mutual recursion.
//
// Source: https://github.com/AeneasVerif/aeneas/blob/main/tests/src/mutually-recursive-traits.rs
// (marked `known-failure` for Lean backend)

pub trait Trait1 {
    type T: Trait2;
}

pub trait Trait2: Trait1 {}

// A second, simpler form of mutual recursion at the trait-bound level:
pub trait T1<T: T2<Self>>: Sized {}
pub trait T2<T: T1<Self>>: Sized {}

pub struct Concrete;

impl Trait1 for Concrete {
    type T = ConcreteT2;
}

pub struct ConcreteT2;

impl Trait2 for ConcreteT2 {}

impl Trait1 for ConcreteT2 {
    type T = ConcreteT2;
}

pub fn trigger_mutually_recursive_traits() {
    let _: <Concrete as Trait1>::T = ConcreteT2;
}
