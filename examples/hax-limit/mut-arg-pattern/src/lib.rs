// Hax limitation: destructuring a `&mut` in a function parameter pattern
// (non-trivial `&mut` input pattern) is not supported.
//
// Hax error: HAX0011 (NonTrivialAndMutFnInput)
// "&mut inputs should be trivial patterns"
// "The support in hax of function with one or more inputs of type `&mut _`
//  is limited. Only trivial patterns are allowed there:
//  `fn f(x: &mut (T, U)) ...` is allowed while `f((x, y): &mut (T, U))`
//  is rejected."
//
// Source: hax-types/src/diagnostics/mod.rs, Kind::NonTrivialAndMutFnInput = 11
// Issue: https://github.com/hacspec/hax/issues/1405
//   "Unsupported Rust: deep mutable borrow on function input"
//   labels = [keep-open, unsupported-rust]

fn fill_pair((a, b): &mut (u32, u32), val: u32) {
    *a = val;
    *b = val + 1;
}

pub fn hax_limit_mut_arg_pattern() -> (u32, u32) {
    let mut pair = (0u32, 0u32);
    fill_pair(&mut pair, 42);
    pair
}
