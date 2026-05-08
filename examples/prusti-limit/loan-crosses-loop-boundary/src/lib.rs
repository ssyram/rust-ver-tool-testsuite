// Prusti limitation: loans (borrows) that cross a loop boundary are not
// supported.
//
// The Prusti user guide's loop section explicitly lists:
//   "Loans that cross a loop boundary (e.g. loans defined outside the loop,
//    expiring in the loop) | Not supported yet"
//
// When a mutable borrow is created before a loop and used or expired inside
// the loop body, Prusti's fold-unfold algorithm cannot track the permission
// correctly and panics or produces an internal error.
//
// Source: https://viperproject.github.io/prusti-dev/user-guide/verify/loop.html
//         (see the feature-support table)
//         https://github.com/viperproject/prusti-dev/issues/543
//         (entries: borrow_in_guard.rs)

/// Fill every element of `buf` with `value` by holding a single mutable borrow
/// across loop iterations. The outer `&mut` reference is a loan that starts
/// before the loop and expires after it — crossing the loop boundary.
pub fn fill(buf: &mut [u8], value: u8) {
    let mut i = 0usize;
    while i < buf.len() {
        buf[i] = value;
        i += 1;
    }
}

pub fn trigger_loan_crosses_loop_boundary() {
    let mut buf = [0u8; 4];
    fill(&mut buf, 255);
}
