# float-types

**Limitation:** Aeneas does not support floating-point types (`f32`, `f64`).
Any use of float literals, float arithmetic, or enums/structs containing float
fields causes translation to fail.

## Trigger

```rust
pub fn scale(x: f64, factor: f64) -> f64 {
    x * factor
}

pub fn make_measurement() -> Measurement {
    Measurement::Value(1.5)   // float literal in enum variant
}
```

## Aeneas errors

```
[Error] Improperly typed constant value    -- triggered by float literals
[Error] unsupported floats                 -- triggered by float arithmetic
```

## Why

Aeneas maps Rust scalar types to proof-assistant numerics (e.g., `u32 -> U32`,
`i64 -> I64`).  No such mapping exists for `f32`/`f64`: floating-point
arithmetic is not natively supported in F*, Lean, or Coq's logic in a way that
Aeneas's functional translation can exploit, and the LLBC float scalar kind is
simply left unhandled.

## Source

<https://github.com/AeneasVerif/aeneas/issues/828>
