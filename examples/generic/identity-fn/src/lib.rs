/// Plain generic fn with multiple instantiations.
/// Tests monomorphisation vs polymorphic translation.
pub fn generic_identity() {
    fn id<T>(x: T) -> T { x }
    let _a = id::<i32>(42);
    let _b = id::<bool>(true);
    let _c = id::<u8>(7);
}
