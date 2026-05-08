// Hax limitation: `if let ... && let ...` (let chains) are not supported.
//
// Let chains (RFC 2497) allow combining multiple `let` bindings with `&&`
// inside `if`/`while`. Stable since Rust 1.88 in edition 2024.
//
// Hax issue: https://github.com/hacspec/hax/issues/2018
//   "[Unsupported Rust] Let chains"
//   labels = [unsupported-rust]
//   Exact reproduction from the issue body:
//     fn f(x: Option<u32>, y: Option<u32>) -> u32 {
//         if let Some(a) = x && let Some(b) = y { a + b } else { 0 }
//     }

fn both_some(x: Option<u32>, y: Option<u32>) -> u32 {
    if let Some(a) = x && let Some(b) = y {
        a + b
    } else {
        0
    }
}

pub fn hax_limit_let_chains() -> u32 {
    both_some(Some(3), Some(4))
}
