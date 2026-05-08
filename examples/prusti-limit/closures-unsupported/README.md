# closures-unsupported

**Prusti limitation:** Closures are not supported.

Prusti reports `[Prusti: unsupported feature] this is unsupported, because it uses closures`
for any function body that defines or invokes a closure. The specification syntax
for closures (`closure!(...)`) is documented as "NOT YET SUPPORTED" and the
feature is tracked in PR #138.

**Sources:**
- <https://viperproject.github.io/prusti-dev/user-guide/verify/closure.html>
- <https://github.com/viperproject/prusti-dev/issues/169>
