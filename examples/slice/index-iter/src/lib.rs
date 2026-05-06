/// Slice an array, iterate, index.
/// Exercises array → slice coercion, slice iteration, bounds checks.
pub fn slice_index_iter() {
    let arr = [1i32, 2, 3, 4, 5];
    let s: &[i32] = &arr[1..4];
    let mut sum = 0i32;
    for x in s {
        sum += *x;
    }
    let _ = sum + s[0];
}
