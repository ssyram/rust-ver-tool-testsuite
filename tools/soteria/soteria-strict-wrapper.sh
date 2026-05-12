#!/usr/bin/env bash
# soteria-strict-wrapper.sh
#
# Discriminates two kinds of soteria-rust exit≠0 outcomes per
# architecture.md §一 "bug detect 归 SUCCESS"（§四 B 派生）:
#
# 1. Tool's own crash / Charon frontend crash (exit 2/3 or exit 1 with
#    "Thread panicked when extracting"):
#    → keep exit ≠ 0 → FAILED (tool's own failure)
#
# 2. Tool completed symbolic execution + reported a bug in entry code:
#    - stdout line "bug: ..." (soteria-rust's bug self-disclosure)
#    - stdout line "found issues in T, errors in N branch (out of M)"
#    → rewrite exit 1 → exit 0 → SUCCESS (bug-detect is best output)
#
# 3. exit 1 caused by cargo build error / unresolved import (single-file
#    pipeline can't see deps):
#    → keep exit 1 → FAILED (tool capability boundary — no deps support)
#
# Precedence:
#   panic-extracting > bug-detect-signature > default (FAILED)

set -uo pipefail

eval $(opam env --switch=soteria-install) 2>/dev/null || true
export PATH=$PATH:${TS_SOTERIA_BIN_DIR}

soteria-rust exec --rustc=--edition=2021 src/lib.rs 2>&1 | tee /tmp/soteria-$$.log
rc=${PIPESTATUS[0]}

if [[ $rc -eq 0 ]]; then
    rm -f /tmp/soteria-$$.log
    exit 0
fi

# Strip ANSI color codes for robust signature matching (soteria emits
# colored bug headers that prevent plain "^bug:" matching).
SCRUBBED=/tmp/soteria-$$.scrubbed
sed 's/\x1b\[[0-9;]*m//g' /tmp/soteria-$$.log > "$SCRUBBED"

# Priority 1: tool's own crash / extraction panic → FAILED
if grep -q "Thread panicked when extracting" "$SCRUBBED" 2>/dev/null; then
    rm -f /tmp/soteria-$$.log "$SCRUBBED"
    exit $rc
fi

# Priority 2: soteria completed symex + reported bug in user code → SUCCESS
# Two signatures (either suffices):
#   - "bug: <description> in lib::..." — soteria's self-disclosed bug header
#   - "found issues in <time>, errors in N branch (out of M)" — verification summary
if grep -qE "^bug:.*in lib::|found issues in .* errors in [0-9]+ branch" "$SCRUBBED" 2>/dev/null; then
    echo "[soteria-oracle] tool detected bug in entry code → bug-detect SUCCESS per architecture.md §一" >&2
    rm -f /tmp/soteria-$$.log "$SCRUBBED"
    exit 0
fi

# Default: tool's own limitation (cargo build error / unsupported / etc.) → FAILED
rm -f /tmp/soteria-$$.log "$SCRUBBED"
exit $rc
