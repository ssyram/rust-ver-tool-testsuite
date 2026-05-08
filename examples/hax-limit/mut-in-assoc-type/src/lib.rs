// Hax limitation: `&mut T` appearing in an associated type is not supported.
//
// Hax error: HAX0003 (UnallowedMutRef) at the associated type definition site.
// Error message seen in practice:
//   "error: [HAX0003] (DirectAndMut) The mutation of this &mut is not allowed here."
//
// Source: https://github.com/hacspec/hax/issues/1674
//   "Unsupported Rust: `&mut` in associated types"
//   labels = [engine, unsupported-rust, keep-open]
//   Exact reproduction from the issue:
//     trait DoesStuff<T> { type Out; }
//     impl<'a, T> DoesStuff<T> for &'a mut T { type Out = &'a mut T; }

pub trait Transform {
    type Output;
    fn apply(self) -> Self::Output;
}

impl<'a> Transform for &'a mut u32 {
    // `&'a mut u32` in an associated type position — hax rejects this
    type Output = &'a mut u32;
    fn apply(self) -> Self::Output {
        *self += 1;
        self
    }
}

pub fn hax_limit_mut_in_assoc_type() {
    let mut x = 0u32;
    x.apply();
}
