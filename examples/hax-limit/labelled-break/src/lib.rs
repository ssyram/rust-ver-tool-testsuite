// Hax limitation: `break` or `continue` targeting a labelled outer loop/block
// is not supported.
//
// Hax issue: https://github.com/hacspec/hax/issues/1799
//   "Unsupported Rust: break or continue to labelled blocks or loops"
//   References: https://doc.rust-lang.org/std/keyword.break.html
//
// Also issue #1800: "Explicit labels on `break`s (or `continue`s) are not honored"
//   labels = [bug, engine, rust-engine]
//
// The engine's control-flow transformation phases handle break/continue
// but do not correctly propagate explicit label targets.

pub fn hax_limit_labelled_break() -> Option<(usize, usize)> {
    let matrix = [[1u32, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12], [13, 14, 15, 99]];
    let target = 99u32;
    let mut found = None;
    'outer: for i in 0..4 {
        for j in 0..4 {
            if matrix[i][j] == target {
                found = Some((i, j));
                break 'outer;
            }
        }
    }
    found
}
