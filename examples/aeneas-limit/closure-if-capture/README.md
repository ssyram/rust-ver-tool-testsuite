# closure-if-capture

**Limitation:** Aeneas cannot extract a closure that simultaneously captures an
outer variable *and* contains an `if-then-else` expression in its body.

## Trigger

```rust
pub fn make_closure_with_capture_and_branch(a: u64) {
    let _c = || {
        if true { a } else { a }
    };
}
```

## Aeneas error

```
[Error] Unimplemented
... Could not translate the body of function '...::closure...'::call
```

## Why

When Aeneas processes a closure it must generate `Fn`/`FnMut`/`FnOnce` `call`
implementations.  The combination of an outer capture (which introduces an
implicit state field) and an `if-then-else` (which requires branching in the
pure functional model) hits an unimplemented code path in `interp/Interp.ml`.

## Source

<https://github.com/AeneasVerif/aeneas/issues/924>
