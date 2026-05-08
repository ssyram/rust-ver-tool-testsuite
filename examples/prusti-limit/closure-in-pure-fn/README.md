# closure-in-pure-fn

**Prusti limitation:** Closures inside `#[pure]` functions are not supported.

A function marked `#[pure]` (used in Prusti specifications) must not contain
any closure literal or closure call. Even a trivially pure immediately-invoked
closure triggers two errors:

1. `use of impure function "std::ops::Fn::call" in pure code is not allowed`
2. `unsupported constant type ... Closure(...)`

This restriction means that specification helpers cannot be factored through
closures, which limits code reuse in Prusti contracts.

**Sources:**
- <https://github.com/viperproject/prusti-dev/issues/1543>
