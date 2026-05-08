// Creusot limitation: mutual recursion between two functions is unsupported.
// Creusot's termination check does not handle mutually recursive call graphs;
// attempting to translate them causes an internal error or produces an
// unbound symbol in Why3 output.
//
// Source: Issue #910 "Mutual recursion is unsupported"
//         Issue #1232 "Remaining unsoundness in the termination check"
//         (labelled bug/soundness/termination)
//
// cargo check: passes (plain recursive Rust, no overflow for small n)
// Creusot: internal error / Why3 "unbound function symbol" at verification time

fn is_odd(n: u32) -> bool {
    if n == 0 {
        false
    } else {
        is_even(n - 1)
    }
}

pub fn is_even(n: u32) -> bool {
    if n == 0 {
        true
    } else {
        is_odd(n - 1)
    }
}

pub fn trigger_is_even() {
    let _ = is_even(4);
}
