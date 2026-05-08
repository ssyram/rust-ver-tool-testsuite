# loan-crosses-loop-boundary

**Prusti limitation:** Loans (borrows) that cross a loop boundary are not supported.

The Prusti user guide's loop section includes an explicit support table with:

> Loans that cross a loop boundary (e.g. loans defined outside the loop,
> expiring in the loop) | **Not supported yet**

When a mutable borrow is live at the start of a loop and expires (or is
re-borrowed) inside the loop body, Prusti's fold-unfold algorithm loses track
of the fractional permission, leading to internal errors or panic.

**Sources:**
- <https://viperproject.github.io/prusti-dev/user-guide/verify/loop.html>
  (feature support table)
- <https://github.com/viperproject/prusti-dev/issues/543>
  (see `borrow_in_guard.rs` entry)
