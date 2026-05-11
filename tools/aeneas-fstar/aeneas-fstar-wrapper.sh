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
"$CHARON_BIN" cargo --preset=aeneas
echo "[aeneas-fstar-wrapper] charon exit: $?"

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
# 临时关 set -e：set -euo pipefail 下 aeneas 非 0 退出会直接终止脚本，
# 导致下面的诊断行（[aeneas-fstar-oracle] FAIL: ...）丢失。oracle 不漏，但诊断质量降级。
set +e
"$AENEAS_BIN" -backend fstar -dest "$FSTAR_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
set -e
echo "[aeneas-fstar-wrapper] aeneas exit: $AENEAS_EXIT"

if [[ $AENEAS_EXIT -eq 0 ]]; then
    echo "[aeneas-fstar-wrapper] generated fstar files:"
    find "$FSTAR_OUT" -name "*.fst" | sort
else
    echo "[aeneas-fstar-oracle] FAIL: aeneas exit $AENEAS_EXIT" >&2
fi

exit $AENEAS_EXIT
