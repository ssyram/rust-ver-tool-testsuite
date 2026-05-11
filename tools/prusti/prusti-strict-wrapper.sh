#!/usr/bin/env bash
# prusti-strict-wrapper.sh
#
# Closes the §D 产物存在性 check 缺失 gap documented in:
#   docs/fixes/oracle-leak-audit-2026-05-08.md §3.6, §4.4
#   docs/fixes/oracle-leak-rules-implementation-2026-05-08.md §2.2
#
# Background
# ----------
# tools/prusti/README.md §"检测条件" defines:
#   "前端接受：exit code 0 且 target/verify/log/viper_program/*.vpr
#    至少一个文件存在"
# but tools/prusti/tool.toml's command never checked the .vpr existence —
# only the exit code. Per project rule §六-2 (no partial / silent skip), this
# leaves a理论窗口: a future toolchain change could make cargo-prusti exit 0
# while encoder fast-paths skip the actual MIR → Viper lowering, producing
# zero .vpr — and our oracle would not notice.
#
# This wrapper re-asserts the README's documented oracle in code: cargo-prusti
# exit 0 AND ≥ 1 .vpr file produced ⇔ SUCCESS.
#
# Empirical state (run-1778226613-5282)
# -------------------------------------
# All 56 prusti SUCCESS run ≥ 13 s wall, dominated by JVM bootstrap +
# encoder. The encoder branch on the NEW config (PRUSTI_NO_VERIFY=false +
# PRUSTI_DUMP_VIPER_PROGRAM=true + PRUSTI_PRINT_HASH=true) writes .vpr unconditionally
# whenever encoding succeeds (verified in prusti-server/process_verification.rs
# at commit a0681ee). So the rule is currently 0%-误报 and 0%-漏报 in the
# empirical sense.
#
# Reverse-false-positive analysis
# -------------------------------
# Will this rule reject a legitimate SUCCESS?
# - PRUSTI_DUMP_VIPER_PROGRAM=true is the documented mechanism for emitting
#   .vpr after encoder completion (commit a0681ee, prusti-utils/src/config.rs).
# - PRUSTI_PRINT_HASH=true short-circuits AFTER the dump
#   (process_verification_request returns Success after dump, before
#   new_viper_verifier()).
# - Therefore: any path that reaches "exit 0" passes through the dump site
#   first, producing at least one .vpr.
# - The rule rejects only "exit 0 + 0 .vpr" — a state that does not occur
#   under the documented config; reaching it implies an upstream encoder
#   silently fast-pathed without writing the dump, which is exactly the
#   silent-skip we want to surface.
# - No legitimate SUCCESS satisfies the rule's reject condition →
#   0 误报 by construction.

set -uo pipefail

# All env (PATH / JAVA_HOME / PRUSTI_* / RUSTUP_TOOLCHAIN / etc.) is set by
# the calling tool.toml `env` invocation. This script just runs cargo-prusti
# and post-checks artefact existence.

# tool.toml re-exports TS_CARGO_PRUSTI as CARGO_PRUSTI (the runner strips
# all TS_* envvars before spawning children — see runner/src/exec.rs:165).
: "${CARGO_PRUSTI:?CARGO_PRUSTI must be set in env (re-exported via tool.toml)}"

arch -x86_64 "$CARGO_PRUSTI"
rc=$?

if [[ $rc -ne 0 ]]; then
    # cargo-prusti reported a real failure (encoder rejected unsupported
    # feature, internal error, ICE etc.) — propagate as-is.
    exit "$rc"
fi

# exit 0 — assert .vpr existence per README's documented oracle.
# log_dir is target/verify/log/viper_program/ by prusti convention (a0681ee).
log_dir="target/verify/log/viper_program"
vpr_count=$(find "$log_dir" -name '*.vpr' 2>/dev/null | wc -l | tr -d ' ')

if [[ "$vpr_count" -eq 0 ]]; then
    cat >&2 <<EOF
[prusti-oracle] FAIL: cargo-prusti exited 0 but produced 0 .vpr files in
[prusti-oracle]       $log_dir/.
[prusti-oracle]       Under the NEW config (PRUSTI_NO_VERIFY=false +
[prusti-oracle]       PRUSTI_DUMP_VIPER_PROGRAM=true + PRUSTI_PRINT_HASH=true),
[prusti-oracle]       any successful encoder run must write at least one .vpr
[prusti-oracle]       before short-circuiting before Silicon. Zero .vpr means
[prusti-oracle]       the encoder fast-pathed without lowering — silent skip.
[prusti-oracle]       Rejecting per project §六-2 反作弊 (no partial).
[prusti-oracle]       See docs/fixes/oracle-leak-audit-2026-05-08.md §3.6.
EOF
    exit 1
fi

exit 0
