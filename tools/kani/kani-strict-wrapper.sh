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

if [[ -z "$hit" ]]; then
    exit 0
fi

# P37 (§六 当前 crate 焦点宽度过滤): kani 5-marker output only aggregates
# counts without source spans (kani-compiler does not emit per-occurrence
# diagnostics + cargo-kani doesn't accept --message-format=json). We apply
# *reverse evidence*: grep entry crate src/ for the marker-triggering
# keywords. If entry crate src/ contains none of them, the markers must
# originate from deps (std / cargo registry / vendor) — per principles.md
# §六 当前 crate 焦点 (宽度切割), external-dep partial does not count →
# suppress FAILED → SUCCESS. If entry crate src/ self-uses any triggering
# keyword → keep FAILED.

python3 - "$out_file" <<'PYEOF'
import re, sys, os

# Map each kani marker to entry-source keyword patterns that trigger it.
# Patterns are conservative: false positives (extra FAILED) are preferred
# over false negatives (missed entry self-use).
patterns = {
    'TerminatorKind::InlineAsm': r'\b(asm|global_asm)\s*!',
    'simd_cast': r'\b(simd_cast|simd_eq|simd_xchg|simd_extract|simd_insert|simd_add|simd_mul|simd_sub|simd_and|simd_or|simd_xor|simd_shl|simd_shr|simd_div|simd_rem|simd_neg|simd_select)\b|\bcore::simd\b|\bstd::simd\b',
    'catch_unwind': r'\b(catch_unwind|catch_panic)\b',
    'ptr_mask': r'(::mask\s*\(|\bptr::mask\b|\bpointer::mask\b)',
    'C string literal': r'(\bc"|\bcr"|CStr::from_bytes_with_nul|\bcstr!|\bc_str!)',
}

with open(sys.argv[1]) as fp:
    kani_text = fp.read()

fired = []
for marker in patterns:
    if re.search(r'^\s+-\s+' + re.escape(marker) + r'\b', kani_text, re.M):
        fired.append(marker)

src_text = ''
for root, dirs, files in os.walk('src'):
    dirs[:] = [d for d in dirs if not d.startswith('.') and d != 'target']
    for f in files:
        if f.endswith('.rs'):
            try:
                with open(os.path.join(root, f), errors='ignore') as fp:
                    src_text += fp.read() + '\n'
            except OSError:
                pass

entry_self_use = []
for marker in fired:
    if re.search(patterns[marker], src_text):
        entry_self_use.append(marker)

if entry_self_use:
    print(f'[kani-oracle] FAIL: codegen with hard-unsupported markers; entry crate src/', file=sys.stderr)
    print(f'[kani-oracle]       self-uses triggering keyword(s) for: {entry_self_use}', file=sys.stderr)
    print(f'[kani-oracle]       Per project §六-2 反作弊, entry-self partial → FAILED.', file=sys.stderr)
    sys.exit(2)

print(f'[kani-oracle] kani 5-markers fired {fired} but entry crate src/ has no triggering', file=sys.stderr)
print(f'[kani-oracle]       keyword; markers must originate from deps (std/registry/vendor).', file=sys.stderr)
print(f'[kani-oracle]       Per principles.md §六 当前 crate 焦点 (宽度切割), external-dep', file=sys.stderr)
print(f'[kani-oracle]       partial does not count — suppressing FAILED → SUCCESS.', file=sys.stderr)
sys.exit(0)
PYEOF
exit $?
