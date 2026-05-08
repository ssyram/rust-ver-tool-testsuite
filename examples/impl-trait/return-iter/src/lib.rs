/// `impl Trait` in return position.
pub fn impl_trait_iter() {
    fn make_iter(n: u32) -> impl Iterator<Item = u32> {
        (0..n).map(|x| x * 2)
    }
    let it = make_iter(5);
    let _: u32 = it.sum();
}
