# shallow-borrow-match-guard

**Prusti limitation:** Shallow borrows are not supported.

When a `match` expression includes a guard (`if` clause) on an arm that binds a
field from a reference-typed scrutinee, the Rust compiler lowers this into a
"shallow borrow" of the outer reference. Prusti cannot encode such borrows and
reports:

```
[Prusti: unsupported feature]
unsupported creation of shallow borrows (implicitly created when lowering matches)
```

**Sources:**
- <https://github.com/viperproject/prusti-dev/issues/1388>
- <https://github.com/viperproject/prusti-dev/issues/543>
