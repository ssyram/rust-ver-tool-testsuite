# return-inside-nested-loop

**Limitation:** Aeneas does not support `return` from inside a nested loop, nor
`break`/`continue` with an outer-loop label.  This is explicitly documented in
the README.

## Trigger

```rust
pub fn first_even(xs: &[i32]) -> Option<i32> {
    for &x in xs {
        if x % 2 == 0 {
            return Some(x);   // return from inside a for-loop body
        }
    }
    None
}

pub fn outer_break_label() {
    'outer: loop {
        loop {
            break 'outer;    // break to labelled outer loop
        }
    }
}
```

## Aeneas error

```
[Error] Unreachable
... Could not translate the body of function ...
```

## Why

Aeneas models loops as recursive functions.  A `return` (or labelled break) from
an *inner* loop must propagate through one or more recursive calls to an *outer*
continuation, which requires generating early-exit machinery that the current
symbolic interpreter does not implement.

## Source

README Limitations section: <https://github.com/AeneasVerif/aeneas>  
Issue: <https://github.com/AeneasVerif/aeneas/issues/822>
