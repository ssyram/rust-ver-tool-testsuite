# trait-impl-mut-param-mismatch

**Limitation:** Instantiating a generic trait with `&mut T` as a type argument
causes a mismatch between the abstract trait model and the concrete impl model
in Aeneas's extracted Lean code.

## Trigger

```rust
pub trait Process<Arg> {
    fn run(arg: Arg);
}

impl Process<&mut u8> for Worker {
    fn run(arg: &mut u8) { *arg += 1; }
}
```

## Aeneas-generated (wrong) Lean snippet

The trait record expects:

```lean
f : Arg → Result Unit
```

But the impl at `&mut u8` generates:

```lean
f : Std.U8 → Result Std.U8   -- backward function for the mutable ref
```

The types don't unify, so the impl cannot satisfy the trait dictionary.

## Why

Aeneas's "backward function" transformation adds an extra output for each mutable
reference in function signatures.  When `Arg = &mut u8` is substituted, the impl
method's signature is extended with the loan-giving return value, but the
*abstract* trait definition was not extended in the same way, producing
incompatible types.

## Source

<https://github.com/AeneasVerif/aeneas/issues/961>
