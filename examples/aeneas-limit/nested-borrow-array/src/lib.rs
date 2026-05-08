// Aeneas limitation: nested borrows formed by placing a reference into an array,
// then indexing that array to assign back into an Option<&T>.
//
// Aeneas errors: "[Error] Found a case of unsupported nested borrows"
// when it encounters the pattern `arr[i]` where arr is `[&T; N]`.
// The reference inside the array creates a borrow that is "nested" under the
// array's own borrow, and Aeneas's symbolic execution cannot handle this
// configuration.
//
// Source: https://github.com/AeneasVerif/aeneas/issues/929

pub struct H<T>(pub T);

impl<T> H<T> {
    pub fn find_max(&self) -> Option<&T> {
        let mut max: Option<&T> = None;
        let arr = [&self.0];
        for i in 0..1usize {
            max = Some(arr[i]);
        }
        max
    }
}

pub fn trigger_nested_borrow_array() {
    let h = H(42u32);
    let _ = h.find_max();
}
