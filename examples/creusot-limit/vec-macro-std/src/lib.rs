// Creusot limitation: the standard `vec![...]` macro uses compiler "magic"
// (a special `box_free` / `RawVec` intrinsic path) to directly initialize
// heap memory.  Creusot cannot translate this magic and will crash on any
// function that calls the standard-library `vec!` macro.
// Creusot-std ships a replacement macro `creusot_std::std::vec::vec` with
// the same semantics but without the intrinsic, which Creusot users must
// import explicitly.
//
// Source: guide/src/limitations.md – "vec! macro" section
//         "Creusot does not support this magic, hence the version provided
//          by the standard library is not supported in Creusot."
//
// cargo check: passes (standard Rust)
// Creusot: translation error / crash when processing vec! desugaring

pub fn make_vec() -> Vec<u32> {
    vec![1, 2, 3]
}
