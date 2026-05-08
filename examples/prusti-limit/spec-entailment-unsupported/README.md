# spec-entailment-unsupported

**Prusti limitation:** Specification entailments (higher-order function contracts) are not supported.

Prusti defines a `f |= |args| [requires(...), ensures(...)]` syntax for
constraining what contract a closure or function-pointer argument must satisfy.
The feature is documented in the user guide but is explicitly marked
"NOT YET SUPPORTED". This means that higher-order functions whose correctness
depends on properties of their callback cannot be verified in Prusti today.

The entry also demonstrates `for<'a> Fn(...)` (HRTB) bounds, which interact
with the same unsupported specification machinery.

**Sources:**
- <https://viperproject.github.io/prusti-dev/user-guide/verify/spec_ent.html>
- <https://viperproject.github.io/prusti-dev/user-guide/verify/closure.html>
