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
set +e
"$CHARON_BIN" cargo --preset=aeneas 2>charon_stderr.log
CHARON_EXIT=$?
set -e
cat charon_stderr.log >&2

# Gate (R7 2026-05-12 by 不公信 / Oracle 不冤枉): charon may exit 0 yet emit
# silent failure signals on stderr:
#   - "is not supported" → charon dropped or opaque-fied a construct
#     (e.g. inline asm, branching Box init) → aeneas sees degraded .llbc
#     and produces an .lean that drops the affected entry semantics (D3.2)
#   - "^error:" with exit 0 → charon's own type checker rejected the
#     program but failed to propagate the exit code (D3.3, e.g.
#     "Type error after transformations: Found incorrect clause var")
# Both are silent partials we must surface as FAILED. See D3.2 / D3.3 in
# docs/fixes/decisions-2026-05-11.md.
if [[ $CHARON_EXIT -eq 0 ]] && grep -qE "is not supported|^error:" charon_stderr.log; then
    echo "[aeneas-lean-oracle] FAIL: charon exited 0 but emitted partial-signal stderr ('is not supported' or '^error:'); silent degradation prevented" >&2
    rm -f charon_stderr.log
    exit 1
fi
rm -f charon_stderr.log

if [[ $CHARON_EXIT -ne 0 ]]; then
    echo "[aeneas-lean-wrapper] charon failed: exit $CHARON_EXIT" >&2
    exit $CHARON_EXIT
fi
echo "[aeneas-lean-wrapper] charon exit: $CHARON_EXIT"

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
# 同时捕获 aeneas stderr+stdout 到 log（合流），便于 gate 检测 Warn 通道 partial。
set +e
"$AENEAS_BIN" -backend lean -dest "$LEAN_OUT" "$LLBC_FILE" 2>&1 | tee aeneas_stage2.log
AENEAS_EXIT=${PIPESTATUS[0]}
set -e
echo "[aeneas-lean-wrapper] aeneas exit: $AENEAS_EXIT"

# Gate (R7 2026-05-12 by 不公信 / Oracle 不冤枉): aeneas exit 0 不充分。
# aeneas 在 Warn 通道（[Warn] 行）输出 partial 自陈而非 craise → exit 0
# + 产物落盘但产物不可用。v6 cc-route 漏报审查锁定 4 类自陈：
#   - "model will not type-check" → mutually-recursive trait / impl partial
#   - "generated code will likely be incorrect" → associated type partial
#   - "seems to be missing the corresponding field" → Lean builtin model 缺
#   - "could not find the information for item" → core trait method silent drop
# 任一命中 → FAILED。aeneas README 第 36/44 行"形式可证 0 漏报"自陈在
# v6 cc 阶段被推翻，本 gate 恢复诚实。
if grep -qE "model will not type-check|generated code will likely be incorrect|seems to be missing the corresponding field|could not find the information for item" aeneas_stage2.log; then
    echo "[aeneas-lean-oracle] FAIL: aeneas exit 0 but Warn-channel partial self-disclosure ('model will not type-check' / 'generated code will likely be incorrect' / 'seems to be missing the corresponding field' / 'could not find the information for item')" >&2
    rm -f aeneas_stage2.log
    exit 1
fi
rm -f aeneas_stage2.log

if [[ $AENEAS_EXIT -eq 0 ]]; then
    echo "[aeneas-lean-wrapper] generated lean files:"
    find "$LEAN_OUT" -name "*.lean" | sort
else
    echo "[aeneas-lean-oracle] FAIL: aeneas exit $AENEAS_EXIT" >&2
fi

exit $AENEAS_EXIT
