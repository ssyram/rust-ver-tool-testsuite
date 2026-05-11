#!/usr/bin/env bash
# aeneas-lean-wrapper.sh
# Two-stage pipeline: charon → .llbc → aeneas → .lean
#
# Usage: aeneas-lean-wrapper.sh [cwd]
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

echo "[aeneas-lean-wrapper] cwd: $(pwd)"

# ── Stage 1: charon ──────────────────────────────────────────────────────────
echo "[aeneas-lean-wrapper] stage 1: charon cargo --preset=aeneas"
"$CHARON_BIN" cargo --preset=aeneas
echo "[aeneas-lean-wrapper] charon exit: $?"

# Find the produced .llbc file (charon writes <crate_name>.llbc in cwd).
LLBC_FILE="$(ls *.llbc 2>/dev/null | head -1)"
if [[ -z "$LLBC_FILE" ]]; then
    echo "[aeneas-lean-wrapper] ERROR: no .llbc file found after charon" >&2
    exit 1
fi
echo "[aeneas-lean-wrapper] found llbc: $LLBC_FILE"

# ── Stage 2: aeneas ──────────────────────────────────────────────────────────
# 按"不允许 partial"精神：aeneas exit 0 = 完整翻译完成；exit ≠ 0 = partial 或
# panic，无论产物是否落盘一律 FAILED。
# - exit 0：完整翻译，所有 fn / type 都成功
# - exit 1 + "Generated the partial file" → partial（aeneas 自陈"因 N 个 errors
#   生成了部分文件"），按精神 FAILED
# - exit 2：OCaml panic，无产物，FAILED
LEAN_OUT="$(pwd)/lean-out"
mkdir -p "$LEAN_OUT"
echo "[aeneas-lean-wrapper] stage 2: aeneas -backend lean"
# 临时关 set -e：set -euo pipefail 下 aeneas 非 0 退出会直接终止脚本，
# 导致下面的诊断行（[aeneas-lean-oracle] FAIL: ...）丢失。oracle 不漏，但诊断质量降级。
set +e
"$AENEAS_BIN" -backend lean -dest "$LEAN_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
set -e
echo "[aeneas-lean-wrapper] aeneas exit: $AENEAS_EXIT"

if [[ $AENEAS_EXIT -eq 0 ]]; then
    echo "[aeneas-lean-wrapper] generated lean files:"
    find "$LEAN_OUT" -name "*.lean" | sort
else
    echo "[aeneas-lean-oracle] FAIL: aeneas exit $AENEAS_EXIT" >&2
fi

exit $AENEAS_EXIT
