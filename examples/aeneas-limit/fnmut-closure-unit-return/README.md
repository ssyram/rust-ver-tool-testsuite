# fnmut-closure-unit-return

**Limitation:** Aeneas generates incorrect Lean code when an `FnMut` closure
returns `()`.  The binder for `call_mut` is emitted without destructuring the
`(Unit × Closure)` pair, causing a type-checking failure in the output.

## Trigger

```rust
pub fn apply_twice<P: FnMut(usize)>(mut p: P) {
    p(0);
    p(1);
}
```

## Aeneas-generated (wrong) Lean snippet

```lean
let p1 ← coreopsfunctionFnMutPTupleUsizeTupleInst.call_mut p 0#usize
-- p1 has type (Unit × P), not P
```

Should be:

```lean
let (_, p1) ← coreopsfunctionFnMutPTupleUsizeTupleInst.call_mut p 0#usize
```

## Why

`FnMut::call_mut` returns a tuple `(output, updated_state)`.  When `output = ()`,
Aeneas's binder-generation code omits the destructuring step, binding the whole
tuple to `p1` and later passing it where only the updated closure is expected.

## Source

<https://github.com/AeneasVerif/aeneas/issues/960>
