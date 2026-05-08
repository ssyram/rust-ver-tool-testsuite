// Hax limitation: returning `&mut T` from a function is forbidden.
//
// Hax error: HAX0003 (UnallowedMutRef)
// "The mutation of this &mut is not allowed here."
//
// Source: README "Supported Subset" section:
//   "mutable references (aka `&mut T`) on return types or when aliasing are forbidden"
// Issue: https://github.com/hacspec/hax/issues/420
//   "We disallow: 1. defining &mut-returning functions"
//
// The phase `phase_direct_and_mut.ml` raises `UnallowedMutRef` when the
// return type contains `&mut T` (see the `TRef { mut = Mutable }` arm
// that calls `Error.raise { kind = UnallowedMutRef; span }`).

fn get_mut_first(v: &mut [u32]) -> &mut u32 {
    &mut v[0]
}

pub fn hax_limit_ret_mut_ref() {
    let mut data = [10u32, 20, 30];
    let r = get_mut_first(&mut data);
    *r = 99;
}
