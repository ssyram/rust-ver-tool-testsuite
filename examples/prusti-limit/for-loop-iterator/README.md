# for-loop-iterator

**Prusti limitation:** `for`-loops over iterators are not supported.

Rust's `for x in iter` syntax desugars into repeated calls to
`Iterator::next`, which involves borrow-expiry patterns (magic wands) that
Prusti cannot encode in loop invariants. The tests `for_iter.rs` and
`simple_iterator.rs` are both permanently disabled (`ignore-test`) in the
Prusti test suite for this reason.

Any function using `for x in collection` — including standard slice iteration,
range loops over non-trivial ranges, or iterator adapter chains — falls into
this category.

**Sources:**
- <https://github.com/viperproject/prusti-dev/issues/543>
  (see "magic wands in loop invariants" entries: `for_iter.rs`, `simple_iterator.rs`)
