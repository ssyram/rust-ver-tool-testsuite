#!/usr/bin/env bash
# aeneas-fstar-wrapper.sh
# Two-stage pipeline: charon → .llbc → aeneas → .fst
#
# Usage: aeneas-fstar-wrapper.sh [cwd]
#   If [cwd] is given, cd into it first.
#   Otherwise, use the current directory (which is set by the runner).
#
# The runner calls this script with cwd = the example crate root, so no
# argument is needed when invoked through tool.toml.  The optional cwd
# argument is provided for standalone/debugging use.

set -euo pipefail

# CHARON_BIN / AENEAS_BIN come from the runner's environment (sourced from .env).
: "${CHARON_BIN:?CHARON_BIN must be set in .env}"
: "${AENEAS_BIN:?AENEAS_BIN must be set in .env}"

# aeneas is an OCaml binary that needs the opam environment to find
# dynamic libraries (in particular zarith / gmp).  Source the opam env.
# We evaluate it inline rather than require the caller to pre-activate it.
if command -v opam >/dev/null 2>&1; then
    eval "$(opam env --set-switch=default 2>/dev/null)" || true
fi

# Optionally cd into a specified working directory.
if [[ "${1:-}" != "" ]]; then
    cd "$1"
fi

echo "[aeneas-fstar-wrapper] cwd: $(pwd)"

# ── Stage 1: charon ──────────────────────────────────────────────────────────
echo "[aeneas-fstar-wrapper] stage 1: charon cargo --preset=aeneas"
set +e
"$CHARON_BIN" cargo --preset=aeneas 2>charon_stderr.log
CHARON_EXIT=$?
set -e
cat charon_stderr.log >&2

# Gate (R7 2026-05-12 by 不公信 / Oracle 不冤枉): charon may exit 0 yet emit
# silent failure signals on stderr ("is not supported" → opaque-fied
# construct → silent partial; "^error:" with exit 0 → charon type error not
# propagated). See D3.2 / D3.3 in docs/fixes/decisions-2026-05-11.md.
if [[ $CHARON_EXIT -eq 0 ]] && grep -qE "is not supported|^error:" charon_stderr.log; then
    echo "[aeneas-fstar-oracle] FAIL: charon exited 0 but emitted partial-signal stderr ('is not supported' or '^error:'); silent degradation prevented" >&2
    rm -f charon_stderr.log
    exit 1
fi
rm -f charon_stderr.log

if [[ $CHARON_EXIT -ne 0 ]]; then
    echo "[aeneas-fstar-wrapper] charon failed: exit $CHARON_EXIT" >&2
    exit $CHARON_EXIT
fi
echo "[aeneas-fstar-wrapper] charon exit: $CHARON_EXIT"

# Find the produced .llbc file (charon writes <crate_name>.llbc in cwd).
LLBC_FILE="$(ls *.llbc 2>/dev/null | head -1)"
if [[ -z "$LLBC_FILE" ]]; then
    echo "[aeneas-fstar-wrapper] ERROR: no .llbc file found after charon" >&2
    exit 1
fi
echo "[aeneas-fstar-wrapper] found llbc: $LLBC_FILE"

# ── Stage 2: aeneas ──────────────────────────────────────────────────────────
# 按"不允许 partial"精神：exit 0 = 完整翻译；exit ≠ 0 = FAILED（partial / panic）
FSTAR_OUT="$(pwd)/fstar-out"
mkdir -p "$FSTAR_OUT"
echo "[aeneas-fstar-wrapper] stage 2: aeneas -backend fstar"
# 临时关 set -e + tee 合流 stderr/stdout → log，便于 Warn 通道 partial 检测。
set +e
"$AENEAS_BIN" -backend fstar -dest "$FSTAR_OUT" "$LLBC_FILE" 2>&1 | tee aeneas_stage2.log
AENEAS_EXIT=${PIPESTATUS[0]}
set -e
echo "[aeneas-fstar-wrapper] aeneas exit: $AENEAS_EXIT"

# Gate (R7 2026-05-12, Warn 通道 partial 自陈封堵): aeneas exit 0 + 4 类
# Warn 自陈 → FAILED。详 aeneas-coq / aeneas-lean wrapper 同 gate。
if grep -qE "model will not type-check|generated code will likely be incorrect|seems to be missing the corresponding field|could not find the information for item" aeneas_stage2.log; then
    echo "[aeneas-fstar-oracle] FAIL: aeneas exit 0 but Warn-channel partial self-disclosure" >&2
    rm -f aeneas_stage2.log
    exit 1
fi
rm -f aeneas_stage2.log

if [[ $AENEAS_EXIT -eq 0 ]]; then
    echo "[aeneas-fstar-wrapper] generated fstar files:"
    find "$FSTAR_OUT" -name "*.fst" | sort
else
    echo "[aeneas-fstar-oracle] FAIL: aeneas exit $AENEAS_EXIT" >&2
fi

exit $AENEAS_EXIT
