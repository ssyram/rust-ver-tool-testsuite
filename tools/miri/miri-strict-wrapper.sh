#!/usr/bin/env bash
# miri-strict-wrapper.sh
#
# Discriminates two kinds of MIRI exit≠0 outcomes per architecture.md
# §一 "bug detect 归 SUCCESS"（§四 B 派生）:
#
# 1. MIRI's own capability boundary (tool can't ingest the entry):
#    - "unsupported operation: inline assembly is not supported"
#    - "can't call foreign function 'X' on OS 'Y'"
#    - "not available when isolation is enabled" (network etc.)
#    → keep exit ≠ 0 → FAILED
#
# 2. MIRI completed interpretation + detected a real UB in entry code:
#    - "error: Undefined Behavior: reading memory ..."
#    - "error: Undefined Behavior: trying to retag from ..."
#    - etc.
#    → rewrite to exit 0 → SUCCESS (tool worked, bug-detect is best output)
#
# Precedence: "unsupported operation" takes precedence — a single run can
# emit both signatures if MIRI hits unsupported before reaching the UB site;
# in that case MIRI didn't actually finish interpretation, so FAILED is
# correct. Only when there is NO "unsupported" signal but there IS a
# Undefined-Behavior diagnostic do we rewrite to SUCCESS.

set -uo pipefail

cargo +nightly miri run --bin __ts_harness > /tmp/miri-$$.stdout 2> /tmp/miri-$$.stderr
rc=$?
cat /tmp/miri-$$.stdout
cat /tmp/miri-$$.stderr >&2

if [[ $rc -eq 0 ]]; then
    rm -f /tmp/miri-$$.stdout /tmp/miri-$$.stderr
    exit 0
fi

# Priority 1: MIRI's own capability boundary → FAILED (keep rc)
if grep -qE "unsupported operation|can't call foreign function|not available when isolation is enabled" /tmp/miri-$$.stderr 2>/dev/null; then
    rm -f /tmp/miri-$$.stdout /tmp/miri-$$.stderr
    exit $rc
fi

# Priority 2: MIRI finished interpretation + reported UB in user code → SUCCESS
if grep -q "Undefined Behavior" /tmp/miri-$$.stderr 2>/dev/null; then
    echo "[miri-oracle] MIRI completed interpretation and reported UB in entry code → bug-detect SUCCESS per architecture.md §一" >&2
    rm -f /tmp/miri-$$.stdout /tmp/miri-$$.stderr
    exit 0
fi

rm -f /tmp/miri-$$.stdout /tmp/miri-$$.stderr
exit $rc
