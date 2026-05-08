/// Const generic over array length `[T; N]`.
pub fn const_generic_array() {
    fn first<T: Copy, const N: usize>(arr: [T; N]) -> Option<T> {
        if N == 0 { None } else { Some(arr[0]) }
    }
    let _a: Option<i32> = first::<i32, 3>([1, 2, 3]);
    let _b: Option<u8> = first::<u8, 0>([]);
}
