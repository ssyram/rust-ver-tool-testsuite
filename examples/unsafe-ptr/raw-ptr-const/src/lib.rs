/// `const X: *const () = N as _;` and matching against it — disguised
/// integer in a raw-pointer const, plus pattern match equality on raw
/// pointers. Charon's known-failure (pointers-in-consts).
pub fn raw_ptr_const_match() {
    const DISGUISED_INT: *const () = 42 as *const ();

    let p = 43 as *const ();
    let label = match p {
        DISGUISED_INT => 1,
        _ => 0,
    };
    let _ = label;
}
