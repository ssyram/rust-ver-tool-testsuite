// Hax limitation: aliasing a `&mut T` variable is forbidden.
//
// Hax error: HAX0003 (UnallowedMutRef)
// "The mutation of this &mut is not allowed here."
//
// Source: https://github.com/hacspec/hax/issues/420
//   "We disallow: 2. aliasing an &mut-typed variable
//    (i.e. `fn f(x: &mut u8) { let y = x; ...}`)"
//
// The phase `phase_direct_and_mut.ml` raises `UnallowedMutRef` when it
// encounters a mutable reference not in a recognised safe pattern.

fn increment_via_alias(x: &mut u8) -> u8 {
    let y = x;
    *y += 1;
    *y
}

pub fn hax_limit_mut_ref_alias() {
    let mut val = 41u8;
    let _ = increment_via_alias(&mut val);
}
