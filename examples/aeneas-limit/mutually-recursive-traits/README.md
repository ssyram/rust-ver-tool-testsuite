# mutually-recursive-traits

**Limitation:** Aeneas cannot handle associated types inside mutually-recursive
trait declarations.  The standard lifting of associated types to trait parameters
breaks down when the trait cycle introduces ordering constraints that cannot be
resolved linearly.

## Trigger

```rust
pub trait Trait1 {
    type T: Trait2;   // associated type constrained by Trait2
}

pub trait Trait2: Trait1 {}  // Trait2 requires Trait1
```

## Aeneas error

```
[Error] Found an associated type in a trait declaration; trait associated types
are usually lifted to become parameters of the trait definition, but this can
fail with mutually-recursive traits as well as GATs.  Aeneas cannot handle such
types today, and the generated code will likely be incorrect.
Compiler source: symbolic/SymbolicToPure.ml, line 209
```

## Why

Aeneas lifts `type T` to a type parameter of the enclosing trait.  In a mutually
recursive cycle (`Trait1 <-> Trait2`) the dependency graph has no topological
ordering, so the parameter-lifting pass in `SymbolicToPure.ml` cannot determine
where to place the parameter.

## Source

<https://github.com/AeneasVerif/aeneas/blob/main/tests/src/mutually-recursive-traits.rs>
(test is marked `known-failure` for the Lean backend)
