// Prusti limitation: shallow borrows are not supported.
//
// When Rust lowers a `match` expression that has a guard (`if` condition) on a
// match arm binding a reference, the compiler implicitly creates a "shallow
// borrow" of the scrutinee. Prusti cannot encode this borrow and reports:
//
//   [Prusti: unsupported feature]
//   unsupported creation of shallow borrows (implicitly created when lowering matches)
//
// Source: https://github.com/viperproject/prusti-dev/issues/1388
//         https://github.com/viperproject/prusti-dev/issues/543

pub struct Wrapper {
    pub x: i32,
}

/// Classify the wrapped integer as even or odd.
/// The match arm `if x % 2 == 0` causes Rust to introduce a shallow borrow,
/// which Prusti does not support.
pub fn classify(w: &Wrapper) -> &'static str {
    match w {
        Wrapper { x } if x % 2 == 0 => "even",
        Wrapper { x: _ } => "odd",
    }
}

pub fn trigger_shallow_borrow_match_guard() {
    let w = Wrapper { x: 4 };
    let _ = classify(&w);
}
