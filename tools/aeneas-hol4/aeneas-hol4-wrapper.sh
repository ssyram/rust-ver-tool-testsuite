#!/usr/bin/env bash
# aeneas-hol4-wrapper.sh
# Two-stage pipeline: charon → .llbc → aeneas → .sml (HOL4)
#
# Usage: aeneas-hol4-wrapper.sh [cwd]
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

echo "[aeneas-hol4-wrapper] cwd: $(pwd)"

# ── Stage 1: charon ──────────────────────────────────────────────────────────
echo "[aeneas-hol4-wrapper] stage 1: charon cargo --preset=aeneas"
"$CHARON_BIN" cargo --preset=aeneas
echo "[aeneas-hol4-wrapper] charon exit: $?"

# Find the produced .llbc file (charon writes <crate_name>.llbc in cwd).
LLBC_FILE="$(ls *.llbc 2>/dev/null | head -1)"
if [[ -z "$LLBC_FILE" ]]; then
    echo "[aeneas-hol4-wrapper] ERROR: no .llbc file found after charon" >&2
    exit 1
fi
echo "[aeneas-hol4-wrapper] found llbc: $LLBC_FILE"

# ── Stage 2: aeneas ──────────────────────────────────────────────────────────
# 按"不允许 partial"精神：exit 0 = 完整翻译；exit ≠ 0 = FAILED（partial / panic）
HOL4_OUT="$(pwd)/hol4-out"
mkdir -p "$HOL4_OUT"
echo "[aeneas-hol4-wrapper] stage 2: aeneas -backend hol4"
"$AENEAS_BIN" -backend hol4 -dest "$HOL4_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
echo "[aeneas-hol4-wrapper] aeneas exit: $AENEAS_EXIT"

if [[ $AENEAS_EXIT -eq 0 ]]; then
    echo "[aeneas-hol4-wrapper] generated hol4 files:"
    find "$HOL4_OUT" -name "*.sml" -o -name "*.thy" | sort
fi

exit $AENEAS_EXIT
