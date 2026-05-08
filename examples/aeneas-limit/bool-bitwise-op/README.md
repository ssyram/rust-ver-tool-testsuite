# bool-bitwise-op

**Limitation:** Aeneas does not support bitwise operators (`&`, `|`, `^`) when
applied to `bool` values; it only handles them for integer types.

## Trigger

```rust
pub fn bitwise_and_bool(a: bool, b: bool) -> bool {
    a & b      // non-short-circuit AND on bool
}
```

## Aeneas error

```
[Error] Invalid inputs for binop
```

## Why

In LLBC a `bool & bool` operation is encoded as a `BinOp` whose operand type is
`bool`.  Aeneas's binary-operation extractor only has cases for integer scalar
types, so it rejects the boolean variant with an "invalid inputs" error.  The
logical operators `&&` and `||` (which desugar to `if`) are fine; only the
bitwise forms on `bool` are broken.

## Source

<https://github.com/AeneasVerif/aeneas/issues/965>
