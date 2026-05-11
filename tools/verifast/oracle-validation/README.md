# verifast oracle-validation micro-tests

Two minimal `.rs` files used to validate `tools/verifast/verifast-strict-wrapper.sh`'s vacuous-pass rule by hand. **Not part of the corpus** (no `hirusttest.toml`, never picked up by the matrix runner).

Documented in: [`docs/fixes/oracle-leak-rules-implementation-2026-05-08.md`](../../../docs/fixes/oracle-leak-rules-implementation-2026-05-08.md) §2.1.

## Files

| file | shape | expected verifast behaviour | wrapper verdict |
|---|---|---|---|
| `spec_less_baseline.rs` | `pub fn add_one(x) { x + 1 }`, no annotation | exit 0, "0 errors found (37 statements verified)", **0 verbose lines mentioning user file** | **REJECTED** (exit 2, vacuous pass) |
| `spec_bearing_add_one.rs` | `fn foo() -> i32 //@ req true; //@ ens result == 42; { 42 }` | exit 0, "0 errors found (39 statements verified)", **10 verbose lines mentioning user file** | **ACCEPTED** (exit 0) |

The verbose-line discriminator is robust across the N-stmt range: even though
the spec-bearing case yields N = 39 (overlapping the spec-less corpus baseline
range {37..40} → audit's ≤ 40 N-threshold would have falsely rejected it),
the user-file-mentions count is 10 vs 0 — a clean separator.

## How to reproduce

The wrapper expects the file at `src/lib.rs` (matching the runner's invocation
form). Reproduce by hand:

```bash
# Spec-bearing → wrapper exit 0
mkdir -p /tmp/vf-test-spec/src
cp tools/verifast/oracle-validation/spec_bearing_add_one.rs /tmp/vf-test-spec/src/lib.rs
cd /tmp/vf-test-spec
VERIFAST_BIN=$TS_VERIFAST_BIN bash $PROJECT/tools/verifast/verifast-strict-wrapper.sh
echo "EXIT: $?"   # 0

# Spec-less → wrapper exit 2
mkdir -p /tmp/vf-test-less/src
cp tools/verifast/oracle-validation/spec_less_baseline.rs /tmp/vf-test-less/src/lib.rs
cd /tmp/vf-test-less
VERIFAST_BIN=$TS_VERIFAST_BIN bash $PROJECT/tools/verifast/verifast-strict-wrapper.sh
echo "EXIT: $?"   # 2 (FAIL: vacuous pass — symex executed 0 statements in src/lib.rs)
```

Both directions were validated on 2026-05-08 against verifast 26.01 binary in `.tmp/agents-staging/tool-verifast/install/verifast-26.01/bin/verifast`.
