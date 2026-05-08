/// Generic fn with trait bound `Add<Output = T> + Copy`.
/// Tests where-clause / trait-bound resolution.
pub fn generic_sum_bound() {
    use std::ops::Add;
    fn sum<T: Add<Output = T> + Copy>(xs: &[T], zero: T) -> T {
        let mut acc = zero;
        for x in xs { acc = acc + *x; }
        acc
    }
    let _ = sum::<i32>(&[1, 2, 3], 0);
    let _ = sum::<u32>(&[10, 20], 0);
}
