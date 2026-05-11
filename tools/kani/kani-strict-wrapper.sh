#!/usr/bin/env bash
# kani-strict-wrapper.sh
#
# Closes the §C1 codegen-with-unsupported-stub漏报 gap documented in:
#   docs/fixes/oracle-leak-audit-2-2026-05-11.md §3.1, §4.1
#   docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md §2.1
#
# Background
# ----------
# kani tool.toml runs `cargo kani --only-codegen --bin __ts_harness`. We use
# `--only-codegen` to make kani comparable to the rest of the matrix at the
# *front-end support* layer (every other tool stops at translation). But
# `--only-codegen` exits 0 even when kani-compiler encountered MIR nodes it
# cannot fully translate: it emits a stub for them and prints a warning
#
#     warning: Found the following unsupported constructs:
#                  - <construct> (N)
#              Verification will fail if one or more of these constructs is
#              reachable.
#
# That warning is kani self-disclosing "I did not codegen this construct
# faithfully — only a stub". By project §六-2 反作弊 精神, this is partial
# codegen and must be FAILED.
#
# Why the 5-marker subset (and not the full warning list)
# -------------------------------------------------------
# The warning header has many possible constructs. Two of them appear at high
# frequency in normal corpus entries because std panic and std alloc paths use
# them on every realistic crate:
#
#   - `caller_location`   (60/144 SUCCESS in run-1778226613-5282) — std panic
#                           path; appears on virtually every non-trivial entry
#                           that touches `panic!` / `unwrap` / `assert!`.
#   - `foreign function`  (63/144 SUCCESS) — std alloc path
#                           (`posix_memalign` / `memcpy` / etc.); appears on
#                           every entry that uses heap-allocating containers.
#
# These two are kani's *standard* handling of std internals and not a sign
# that user code triggered a hard-unsupported MIR construct. Rejecting on
# them would cause mass false positives (≥ 40% of SUCCESS turns FAILED for
# generic std-using code).
#
# The remaining 5 markers are construct-specific and only appear when the
# user crate (or its non-std deps) writes MIR that kani cannot codegen:
#
#   1. `TerminatorKind::InlineAsm` — inline assembly (asm!/global_asm!) MIR
#      terminator. kani has no goto-cc semantics for it.
#   2. `simd_cast`                 — packed-SIMD cast intrinsic.
#   3. `catch_unwind`              — panic recovery; kani's unwind model is
#      a stub by design.
#   4. `ptr_mask`                  — raw-pointer bit-mask intrinsic.
#   5. `C string literal`          — `c"..."` raw cstr literal MIR rvalue
#      (Rust 2024 stable feature, kani-compiler does not yet lower).
#
# Empirical evidence (run-1778226613-5282)
# ----------------------------------------
# 8 SUCCESS entries hit one of these 5 markers:
#   - charon-limit/inline-asm/nop_via_asm                    InlineAsm
#   - concurrency/thread-mutex/thread_mutex_join             C string + catch_unwind + ptr_mask
#   - deps-complex/{bigint,chrono,collections}-serde         InlineAsm + simd_cast
#   - deps-complex/error-chain                               catch_unwind + ptr_mask + simd_cast
#   - kani-limit/stack-unwinding/trigger_divide_with_recovery catch_unwind
#   - miri-limit/thread-interleaving-partial/...             C string + catch_unwind + ptr_mask
#
# Reverse-false-positive analysis (合法 SUCCESS → 不命中)
# -----------------------------------------------------
# - hello/basic-hello/hello SUCCESS: stdout only has the Kani version banner,
#   no warning header → not triggered ✓
# - bigint/bigint-arith SUCCESS: stdout has caller_location + foreign function
#   warnings, but neither matches any of the 5 markers → not triggered ✓
# - industrial/rsa/...   SUCCESS: same shape (caller_location + foreign fn) ✓
# - industrial/sha2/...  SUCCESS: same shape ✓
# A legitimate SUCCESS entry, by construction, never emits one of the 5
# hard-unsupported markers — they only appear when codegen consumed a MIR
# construct it cannot model. Hence reject condition is unreachable from real
# success ⇒ 0 误报.
#
# Behaviour notes
# ---------------
# - exit ≠ 0: pass through (cargo kani / kani-compiler reported a real error).
# - exit 0  + ≥ 1 of the 5 markers in stdout: rewrite to exit 2 + stderr
#   diagnostic (FAILED).
# - exit 0  + no 5-marker hit: pass through (real SUCCESS).
# - The full kani output is echoed back to stdout so the runner records it.

set -uo pipefail

out_file="$(mktemp -t kani-stdout.XXXXXX)"
trap 'rm -f "$out_file"' EXIT

# Invoke kani exactly as the original tool.toml did. We deliberately keep the
# argv identical to what was there before — only the post-check is new.
cargo kani --only-codegen --bin __ts_harness >"$out_file" 2>&1
rc=$?

# Echo kani's combined output back so the runner records it.
cat "$out_file"

if [[ $rc -ne 0 ]]; then
    exit "$rc"
fi

# exit 0 — look for the 5 hard-unsupported markers inside the
# `Found the following unsupported constructs:` block. We match the bulleted
# list lines: kani prints each as
#     <spaces>- <construct> (<count>)
# We accept any leading whitespace; the marker name appears verbatim.
hit=$(grep -E '^[[:space:]]+-[[:space:]]+(TerminatorKind::InlineAsm|simd_cast|catch_unwind|ptr_mask|C string literal)\b' "$out_file" 2>/dev/null | head -5)

if [[ -n "$hit" ]]; then
    cat >&2 <<EOF
[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs
[kani-oracle]       (kani self-disclosed via 'Found the following unsupported
[kani-oracle]       constructs:' warning). The matched markers are:
$hit
[kani-oracle]       kani replaced these constructs with stubs ("Verification
[kani-oracle]       will fail if one or more of these constructs is reachable"
[kani-oracle]       — kani's own words) but still exits 0 because --only-codegen
[kani-oracle]       does not invoke CBMC. Per project §六-2 反作弊 (no partial
[kani-oracle]       / silent skip), this is a partial-codegen漏报 and must be
[kani-oracle]       FAILED. The 5 marker subset excludes the std prelude
[kani-oracle]       'caller_location' and 'foreign function' warnings which fire
[kani-oracle]       on most non-trivial entries via std panic/alloc paths.
[kani-oracle]       See docs/fixes/oracle-leak-audit-2-2026-05-11.md §3.1.
EOF
    exit 2
fi

exit 0
