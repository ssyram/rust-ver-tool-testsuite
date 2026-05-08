// Prusti limitation: access to reference-typed struct fields is not supported.
//
// When a struct contains (or transitively contains) a reference-typed field,
// Prusti reports:
//
//   [Prusti: unsupported feature] access to reference-typed fields is not supported
//
// This includes cases where the reference is buried inside a generic wrapper
// (e.g. `Outer<Inner<'a>>`), not just direct `&T` fields.
//
// Source: https://github.com/viperproject/prusti-dev/issues/1342
//         https://github.com/viperproject/prusti-dev/issues/1315

pub struct View<'a> {
    /// A direct reference-typed field — Prusti cannot access this field.
    pub data: &'a [u32],
}

/// Return the length of the slice stored inside `View`.
/// Any attempt to access `v.data` in a Prusti-verified context triggers the
/// "reference-typed fields" unsupported error.
pub fn view_len(v: &View<'_>) -> usize {
    v.data.len()
}

pub fn trigger_ref_typed_struct_field() {
    let data = [1u32, 2, 3];
    let v = View { data: &data };
    let _ = view_len(&v);
}
