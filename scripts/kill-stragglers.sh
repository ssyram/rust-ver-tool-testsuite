#!/bin/bash
# Cascade-kill any leftover verifier subprocess + its full process tree.
# Use after a runner crash / interrupt where children may have escaped.
#
# Why cascade: tools like kani spawn cbmc as a grandchild that does not match
# the parent's process name; a flat `pkill -9 -f kani` leaves cbmc occupying
# CPU. We walk the ppid tree + send to the process group to cover both axes.
set -u

# Verifier-related process name patterns (cmdline substring match).
PATTERNS=(
  cbmc cargo-kani kani-compiler kani-driver
  cargo-prusti prusti-driver prusti-rustc
  cargo-creusot creusot-rustc
  charon-driver charon
  hax-engine cargo-hax
  verifast
  verus
  aeneas
  rocq-of-rust
  kmir
  z3 cvc4 cvc5 alt-ergo why3
)

cascade_kill() {
  local pid="$1"
  # 1. recursively kill all descendants by walking ppid tree
  local children
  children=$(pgrep -P "$pid" 2>/dev/null || true)
  for c in $children; do cascade_kill "$c"; done
  # 2. kill the process group if pid is the leader (-pgid)
  kill -9 -- "-$pid" 2>/dev/null || true
  # 3. kill the pid itself
  kill -9 "$pid" 2>/dev/null || true
}

for pat in "${PATTERNS[@]}"; do
  for pid in $(pgrep -f "$pat" 2>/dev/null || true); do
    cascade_kill "$pid"
  done
done

# Verify
sleep 1
joined=$(IFS='|'; echo "${PATTERNS[*]}")
remaining=$(ps -e -o pid,command | grep -E "($joined)" | grep -v grep | grep -v kill-stragglers || true)
if [ -n "$remaining" ]; then
  echo "[warn] still alive after cascade kill:"
  echo "$remaining"
  exit 1
else
  echo "[ok] all clear"
fi
