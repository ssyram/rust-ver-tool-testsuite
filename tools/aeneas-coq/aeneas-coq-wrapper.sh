#!/usr/bin/env bash
# aeneas-coq-wrapper.sh
# Two-stage pipeline: charon → .llbc → aeneas → .v
#
# Usage: aeneas-coq-wrapper.sh [cwd]
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

echo "[aeneas-coq-wrapper] cwd: $(pwd)"

# ── Stage 1: charon ──────────────────────────────────────────────────────────
echo "[aeneas-coq-wrapper] stage 1: charon cargo --preset=aeneas"
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
    echo "[aeneas-coq-oracle] FAIL: charon exited 0 but emitted partial-signal stderr ('is not supported' or '^error:'); silent degradation prevented" >&2
    rm -f charon_stderr.log
    exit 1
fi
rm -f charon_stderr.log

if [[ $CHARON_EXIT -ne 0 ]]; then
    echo "[aeneas-coq-wrapper] charon failed: exit $CHARON_EXIT" >&2
    exit $CHARON_EXIT
fi
echo "[aeneas-coq-wrapper] charon exit: $CHARON_EXIT"

# Find the produced .llbc file (charon writes <crate_name>.llbc in cwd).
LLBC_FILE="$(ls *.llbc 2>/dev/null | head -1)"
if [[ -z "$LLBC_FILE" ]]; then
    echo "[aeneas-coq-wrapper] ERROR: no .llbc file found after charon" >&2
    exit 1
fi
echo "[aeneas-coq-wrapper] found llbc: $LLBC_FILE"

# ── Stage 2: aeneas ──────────────────────────────────────────────────────────
# 按"不允许 partial"精神：exit 0 = 完整翻译；exit ≠ 0 = FAILED（partial / panic）
COQ_OUT="$(pwd)/coq-out"
mkdir -p "$COQ_OUT"
echo "[aeneas-coq-wrapper] stage 2: aeneas -backend coq"
# 临时关 set -e：set -euo pipefail 下 aeneas 非 0 退出会直接终止脚本，
# 导致下面的诊断行（[aeneas-coq-oracle] FAIL: ...）丢失。oracle 不漏，但诊断质量降级。
set +e
"$AENEAS_BIN" -backend coq -dest "$COQ_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
set -e
echo "[aeneas-coq-wrapper] aeneas exit: $AENEAS_EXIT"

if [[ $AENEAS_EXIT -eq 0 ]]; then
    echo "[aeneas-coq-wrapper] generated coq files:"
    find "$COQ_OUT" -name "*.v" | sort
else
    echo "[aeneas-coq-oracle] FAIL: aeneas exit $AENEAS_EXIT" >&2
fi

exit $AENEAS_EXIT
