#!/usr/bin/env bash
# rocq-of-rust-typecheck-wrapper.sh
#
# Two-stage pipeline: rocq-of-rust translate → .v ; coqc → .vo
#
# Stage 1 (translate): identical to tools/rocq-of-rust — invoke the
#                      rocq-of-rust binary on src/lib.rs to write
#                      rocq_translation/<absolute-source-path>.v
# Stage 2 (typecheck): coqc -R <runtime> RocqOfRust -impredicative-set <product>.v
#
# Oracle (gate 1-6 are inherited from tools/rocq-of-rust, gates 7-9 are new):
#   1-6  identical to tools/rocq-of-rust (exit 0, .v present, > 200 bytes,
#        no failure marker, entry_fn appears as Definition)
#   7.   coqc exits 0 on the .v product
#   8.   .vo file written next to .v product
#   9.   coqc stderr contains no "Error" line
#
# Runtime bootstrap strategy (option a — auto-bootstrap, idempotent):
#   On each invocation, stat 9 core runtime .vo files; if any missing, build
#   them once (seconds). Subsequent invocations skip the rebuild. This avoids
#   per-entry build cost (~< 2 s/entry) while staying self-contained: no
#   manual setup beyond the one-time `opam switch create ror-test` documented
#   in tools/rocq-of-rust-typecheck/README.md.
#
# Env contract (set in .env, expanded by runner before spawn; runner strips
# TS_* before spawn so wrapper sees these as plain names re-exported via
# tool.toml):
#   ROCQ_OF_RUST_BIN                — abspath to rocq-of-rust binary
#   ROCQ_OF_RUST_TOOLCHAIN_SYSROOT  — nightly-2024-12-07 sysroot
#   ROR_TYPECHECK_SWITCH            — opam switch name with Rocq 9.0.0 + deps
#   ROR_RUNTIME_PATH                — abspath to the RocqOfRust runtime dir
#                                     (contains M.v, lib/, links/, simulate/)

set -uo pipefail

# ROCQ_OF_RUST_BIN: if unset, fall back to PATH lookup ("rocq-of-rust") —
# matches tools/rocq-of-rust which never sets a binary path env var.
ROCQ_OF_RUST_BIN="${ROCQ_OF_RUST_BIN:-rocq-of-rust}"
: "${ROCQ_OF_RUST_TOOLCHAIN_SYSROOT:?ROCQ_OF_RUST_TOOLCHAIN_SYSROOT must be set}"
: "${ROR_TYPECHECK_SWITCH:?ROR_TYPECHECK_SWITCH must be set}"
: "${ROR_RUNTIME_PATH:?ROR_RUNTIME_PATH must be set}"

# ── Activate the isolated opam switch holding Rocq 9.0.0 + smpl + hammer
# + coqutil. Per docs/research/ror-runnable-deep-dive-2026-05-11.md §3.
if command -v opam >/dev/null 2>&1; then
    eval "$(opam env --switch="$ROR_TYPECHECK_SWITCH" --set-switch 2>/dev/null)" || true
fi

if ! command -v coqc >/dev/null 2>&1; then
    echo "[ror-typecheck-oracle] FAIL: coqc not on PATH after activating switch '$ROR_TYPECHECK_SWITCH'" >&2
    exit 2
fi

# ── Runtime bootstrap: ensure the 9 core runtime .vo files exist.
# Per ror-runnable-deep-dive §4.1 these are the closure for tier-1 typecheck.
CORE_VO=(
    "M.vo"
    "RocqOfRust.vo"
    "RecordUpdate.vo"
    "lib/lib.vo"
    "lib/simulate/lib.vo"
    "links/M.vo"
    "links/RocqOfRust.vo"
    "simulate/M.vo"
    "simulate/RocqOfRust.vo"
)
needs_build=0
for vo in "${CORE_VO[@]}"; do
    if [[ ! -f "$ROR_RUNTIME_PATH/$vo" ]]; then
        needs_build=1
        break
    fi
done

if [[ $needs_build -eq 1 ]]; then
    echo "[ror-typecheck-wrapper] runtime bootstrap: building 9 core .v in $ROR_RUNTIME_PATH" >&2
    pushd "$ROR_RUNTIME_PATH" >/dev/null
    # Order matters: M before {RocqOfRust, lib/lib}; links before simulate.
    BUILD_ORDER=(
        "RecordUpdate.v"
        "M.v"
        "lib/lib.v"
        "RocqOfRust.v"
        "links/M.v"
        "links/RocqOfRust.v"
        "lib/simulate/lib.v"
        "simulate/M.v"
        "simulate/RocqOfRust.v"
    )
    for src in "${BUILD_ORDER[@]}"; do
        if [[ ! -f "${src%.v}.vo" ]]; then
            echo "[ror-typecheck-wrapper] coqc bootstrap: $src" >&2
            if ! coqc -R . RocqOfRust -impredicative-set "$src" >&2; then
                echo "[ror-typecheck-oracle] FAIL: runtime bootstrap failed at $src" >&2
                popd >/dev/null
                exit 2
            fi
        fi
    done
    popd >/dev/null
fi

# ── Stage 1: rocq-of-rust translate (identical logic to tools/rocq-of-rust).
# Set DYLD_LIBRARY_PATH + PATH from the nightly sysroot so the binary can
# find librustc_driver and call its own rustc --print=sysroot.
SYSROOT="$ROCQ_OF_RUST_TOOLCHAIN_SYSROOT"
export DYLD_LIBRARY_PATH="$SYSROOT/lib"
# Prepend nightly sysroot bin so internal rustc resolution hits the right one,
# but keep the rest of PATH (we need coqc from the opam switch).
export PATH="$SYSROOT/bin:$PATH"

mkdir -p rocq_translation
"$ROCQ_OF_RUST_BIN" translate --path src/lib.rs --output-path rocq_translation 2>rocq_stderr.log
rc=$?
cat rocq_stderr.log >&2
# Gate (R7 2026-05-12, D3.1 tier-1 同步): tier-0 wrapper 已抓 "is not yet
# supported" silent partial (see tools/rocq-of-rust/rocq-of-rust-wrapper.sh
# 同 gate)。tier-1 README 第 43 / 136 行硬声明"tier-1 ⊆ tier-0" + "Stage 1
# 与 tier-0 完全等价 6 道 grep"——v6 cc-route 漏报审查发现 tier-1 漏抄此
# gate，破声明。补此 gate 恢复 tier-1 ⊆ tier-0 不变式。
if grep -q "is not yet supported" rocq_stderr.log; then
    echo "[ror-typecheck-oracle] FAIL: rocq-of-rust emitted 'is not yet supported' warning (silent partial translation; exit 0 but feature dropped from .v product)" >&2
    rm -f rocq_stderr.log
    exit 1
fi
rm -f rocq_stderr.log

if [[ $rc -ne 0 ]]; then
    echo "[ror-typecheck-oracle] FAIL: rocq-of-rust translate exit $rc" >&2
    exit $rc
fi

# Gates 2-6 (mirrors tools/rocq-of-rust/tool.toml):
if ! test -n "$(find rocq_translation -name '*.v' -print -quit)"; then
    echo "[ror-typecheck-oracle] FAIL: no .v product written" >&2
    exit 1
fi
if [[ "$(find rocq_translation -name '*.v' -size 0 | wc -l)" -ne 0 ]]; then
    echo "[ror-typecheck-oracle] FAIL: zero-byte .v product" >&2
    exit 1
fi
if [[ "$(find rocq_translation -name '*.v' -size +200c | wc -l)" -eq 0 ]]; then
    echo "[ror-typecheck-oracle] FAIL: no .v product > 200 bytes" >&2
    exit 1
fi
if grep -rqE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )' rocq_translation; then
    echo "[ror-typecheck-oracle] FAIL: product contains explicit failure marker" >&2
    exit 1
fi
if [[ -n "${TS_ENTRY_FN:-}" ]] && ! grep -rqE "^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]" rocq_translation; then
    echo "[ror-typecheck-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .v products" >&2
    exit 1
fi

# ── Stage 2: coqc typecheck.
# Per ror-runnable-deep-dive §5.1: coqc -R <runtime> RocqOfRust -impredicative-set <product>.v
# The product file name "lib.v" is the rocq-of-rust convention (it mirrors the
# input src/lib.rs path). Run coqc from the product directory so its
# auto-generated lib.glob / lib.vo land alongside the source.
PRODUCT_V="$(find rocq_translation -name '*.v' | head -1)"
if [[ -z "$PRODUCT_V" ]]; then
    echo "[ror-typecheck-oracle] FAIL: lost .v product between stage 1 and stage 2" >&2
    exit 1
fi

PRODUCT_DIR="$(dirname "$PRODUCT_V")"
PRODUCT_BASE="$(basename "$PRODUCT_V")"
PRODUCT_STEM="${PRODUCT_BASE%.v}"

echo "[ror-typecheck-wrapper] coqc -R $ROR_RUNTIME_PATH RocqOfRust -impredicative-set $PRODUCT_V" >&2

coqc_err="$(mktemp -t ror-coqc-err.XXXXXX)"
trap 'rm -f "$coqc_err"' EXIT

( cd "$PRODUCT_DIR" && coqc -R "$ROR_RUNTIME_PATH" RocqOfRust -impredicative-set "$PRODUCT_BASE" ) \
    >&2 2>"$coqc_err"
coqc_rc=$?
cat "$coqc_err" >&2

# Gate 7: coqc exit 0
if [[ $coqc_rc -ne 0 ]]; then
    echo "[ror-typecheck-oracle] FAIL: coqc exit $coqc_rc on $PRODUCT_V" >&2
    exit $coqc_rc
fi

# Gate 8: .vo file produced
if [[ ! -f "$PRODUCT_DIR/$PRODUCT_STEM.vo" ]]; then
    echo "[ror-typecheck-oracle] FAIL: coqc exit 0 but no .vo at $PRODUCT_DIR/$PRODUCT_STEM.vo" >&2
    exit 1
fi

# Gate 9: stderr has no "Error" line (defence in depth — coqc usually
# exits non-zero on errors, but a belt-and-braces grep cheaply closes the
# theoretical exit-0-with-stderr-error path).
if grep -qE '^Error' "$coqc_err" 2>/dev/null; then
    echo "[ror-typecheck-oracle] FAIL: coqc exit 0 but stderr contains Error line" >&2
    exit 1
fi

exit 0
