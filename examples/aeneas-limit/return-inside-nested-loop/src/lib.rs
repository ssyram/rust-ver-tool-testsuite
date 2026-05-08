// Aeneas limitation: `return` inside a nested loop (or `break`/`continue` to
// an outer labelled loop) is not supported.
//
// The README explicitly states: "return inside nested loops, or break/continue
// to outer loops are not supported yet (e.g., `'a: loop { loop { break 'a; } }`)."
//
// Aeneas errors with "[Error] Unreachable" / translation failure when the
// symbolic interpreter encounters the early-exit edge from an inner loop.
//
// Source: https://github.com/AeneasVerif/aeneas (README Limitations section)
//         https://github.com/AeneasVerif/aeneas/issues/822

pub fn first_even(xs: &[i32]) -> Option<i32> {
    for &x in xs {
        if x % 2 == 0 {
            return Some(x); // return from inside a for loop
        }
    }
    None
}

pub fn outer_break_label() {
    // break to an outer labelled loop
    'outer: loop {
        loop {
            break 'outer;
        }
    }
}
