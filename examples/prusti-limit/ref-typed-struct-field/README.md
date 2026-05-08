# ref-typed-struct-field

**Prusti limitation:** Access to reference-typed struct fields is not supported.

When a struct has a field whose type is a reference (or a generic wrapper
containing a reference), Prusti cannot process field accesses inside the
function body and reports:

```
[Prusti: unsupported feature] access to reference-typed fields is not supported
```

This issue appeared frequently in real-world codebase evaluations (e.g. running
Prusti against `chrono`) and is acknowledged in the GitHub issue tracker.

**Sources:**
- <https://github.com/viperproject/prusti-dev/issues/1342>
- <https://github.com/viperproject/prusti-dev/issues/1315>
