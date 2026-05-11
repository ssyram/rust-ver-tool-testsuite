#!/usr/bin/env bash
# verifast-strict-wrapper.sh
#
# Closes the §A vacuous-pass漏报 gap documented in:
#   docs/fixes/oracle-leak-audit-2026-05-08.md §3.1, §4.1
#   docs/fixes/oracle-leak-rules-implementation-2026-05-08.md §2.1
#
# Background
# ----------
# verifast tool.toml runs with `-skip_specless_fns`, which silently skips every
# fn that lacks `//@ req/ens/inv/pred` annotations. Our entire corpus has zero
# such annotations (grep -rE '//@\s*(req|ens|inv|pred)' examples/ → 0 hits), so
# verifast's symbolic-execution engine never touches user code on any of our
# entries. exit 0 in that mode is a vacuous pass: VeriFast loaded its own
# prelude, found no specs to discharge, and quit. The fixed-form summary
#
#     0 errors found (N statements verified)
#
# is therefore unreliable as a discriminator: spec-less observed N ∈ {37..40},
# but a minimal spec-bearing entry (single `//@ req true; //@ ens result == 42;`
# fn) yields N = 39 — overlapping the vacuous range. So N alone cannot
# separate vacuous from real (validated empirically — see oracle-validation/
# below).
#
# Rule (more robust than the audit's ≤ 40 N-threshold)
# ----------------------------------------------------
# Run verifast with `-verbose 1`. Verbose mode prints, for every statement
# the symex actually executes, a line of the form
#
#     <secs>: <source-file-path>(LINE,COL-LINE,COL): <action>
#
# When `-skip_specless_fns` skips a user fn, verifast still verifies its own
# prelude (in <verifast-install>/bin/rust/rust_belt/*.rsspec) but produces ZERO
# verbose lines mentioning the user input file. When the fn is NOT skipped
# (i.e. has at least one `//@ req`), the verbose output contains lines
# mentioning the user file path with actions like "Verifying function 'foo'",
# "Executing statement", "Leak check.".
#
# So the rule is: count verbose lines mentioning the input source path
# (`src/lib.rs`). Zero such lines after exit 0 ⇒ vacuous pass ⇒ FAILED.
#
# Reverse-false-positive analysis
# -------------------------------
# Will this rule reject a legitimate SUCCESS?
# - A user fn that survives -skip_specless_fns must have at least one
#   `//@ req` clause.
# - Once it survives, verifast emits verbose lines for its function-prototype
#   implementation check + every executed statement, all tagged with the
#   source path verifast was given.
# - Therefore: spec-bearing SUCCESS ⇒ ≥ 1 verbose line mentioning the user
#   file. The rule's reject condition (0 mentions) is impossible for a real
#   SUCCESS by construction.
#
# Validation evidence (tools/verifast/oracle-validation/)
# -------------------------------------------------------
# Two micro-tests, each fed to verifast as src/lib.rs (matching the runner's
# invocation form):
#   spec_less_baseline.rs (no spec, single `pub fn add_one`):
#     EXIT 0, 0 verbose lines mentioning src/lib.rs   → wrapper rejects ✓
#   spec_bearing_add_one.rs (`fn foo` with `//@ req true; //@ ens ...`):
#     EXIT 0, 10 verbose lines mentioning src/lib.rs  → wrapper accepts ✓
# Run these by hand from oracle-validation/ — see that directory for details.
#
# Behaviour notes
# ---------------
# - exit ≠ 0: pass through untouched (real verify error / parse error / IR
#   refusal — these are legitimate FAILED).
# - exit 0 + 0 user-file mentions: vacuous pass → rewrite to exit 2 + stderr
#   diagnostic.
# - exit 0 + ≥ 1 user-file mention: pass through (real spec-bearing SUCCESS).
# - The verbose stream is captured to stdout for post-mortem; runner records
#   it. Volume is modest (~110 lines for our prelude).

set -uo pipefail

# tool.toml re-exports TS_VERIFAST_BIN as VERIFAST_BIN (the runner strips
# all TS_* envvars before spawning children — see runner/src/exec.rs:165).
: "${VERIFAST_BIN:?VERIFAST_BIN not set in env (re-exported via tool.toml)}"

# Run verifast in verbose mode so we get per-statement source-path lines.
# Capture combined stdout/stderr to a temp file so we can both echo it back
# (so the runner records it) AND grep for the discriminator signal.
out_file="$(mktemp -t verifast-stdout.XXXXXX)"
trap 'rm -f "$out_file"' EXIT

"$VERIFAST_BIN" \
    -verbose 1 \
    -target macOS \
    -shared \
    -skip_specless_fns \
    -ignore_unwind_paths \
    -disable_overflow_check \
    -read_options_from_source_file \
    src/lib.rs \
    >"$out_file" 2>&1
rc=$?

# Echo verifast's combined output back to stdout so the runner records it.
cat "$out_file"

if [[ $rc -ne 0 ]]; then
    exit "$rc"
fi

# exit 0: count verbose lines mentioning the user source file.
# Format (verifast 26.01, -verbose 1):
#     <secs>s: src/lib.rs(LINE,COL-LINE,COL): <action>
# We anchor on `src/lib.rs(` to avoid spurious matches in any error-path
# strings that might quote the path differently.
user_lines=$(grep -cE 'src/lib\.rs\(' "$out_file" 2>/dev/null || true)
# grep -c can return 0/1/etc. but with `|| true` we always get a number.
user_lines=${user_lines:-0}

if [[ "$user_lines" -eq 0 ]]; then
    cat >&2 <<EOF
[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements in
[verifast-oracle]       src/lib.rs. \`-skip_specless_fns\` is on and this
[verifast-oracle]       entry has no \`//@ req/ens/inv/pred\` annotation, so
[verifast-oracle]       verifast verified only its prelude (in
[verifast-oracle]       <install>/bin/rust/rust_belt/*.rsspec) and exited.
[verifast-oracle]       SUCCESS would not reflect verification of user code;
[verifast-oracle]       rejecting per project §六-2 反作弊 (no partial /
[verifast-oracle]       silent skip).
[verifast-oracle]       See docs/fixes/oracle-leak-audit-2026-05-08.md §3.1
[verifast-oracle]       and the implementation log
[verifast-oracle]       docs/fixes/oracle-leak-rules-implementation-2026-05-08.md §2.1.
EOF
    exit 2
fi

exit 0
