#!/usr/bin/env python3
"""
check-translation-correspondence.py

Quick-and-dirty function-level correspondence check for translation tools
(hax-lean, rocq-of-rust, aeneas-{coq,fstar,lean,hol4}).

Goal: from a SUCCESS Rust crate, extract the entry `pub fn` set + same-crate
adjacency closure, then grep them by name in the translation product.

Usage:
  ./check-translation-correspondence.py \
    --src   <path-to-rust-crate>            # contains src/lib.rs
    --output <path-to-translation-product>  # dir containing .lean / .v / .fst / .sml
    --tool  hax-lean | rocq-of-rust | aeneas-lean | aeneas-coq | aeneas-fstar | aeneas-hol4

This is intentionally regex-based; it is not a precise resolver. It is meant
to give a fast hit/miss signal across 5-10 SUCCESS entries.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Iterable

# Common std method names we never want to chase as adjacency edges.
STD_METHOD_BLOCKLIST = {
    "unwrap", "expect", "clone", "to_string", "to_owned", "into",
    "from", "as_ref", "as_mut", "as_str", "as_bytes",
    "iter", "into_iter", "iter_mut", "next", "collect", "map",
    "filter", "fold", "for_each", "count", "sum", "product",
    "len", "is_empty", "push", "pop", "insert", "remove", "get",
    "split", "trim", "parse", "ok", "err", "and_then", "or_else",
    "unwrap_or", "unwrap_or_else", "unwrap_or_default",
    "checked_add", "checked_sub", "checked_mul", "checked_div",
    "wrapping_add", "wrapping_sub", "wrapping_mul",
    "saturating_add", "saturating_sub",
    "borrow", "borrow_mut", "deref", "deref_mut",
    "new", "default", "with_capacity",
    "send", "recv", "lock", "read", "write",
    "abs", "min", "max", "pow", "sqrt",
    "println", "print", "eprintln", "format", "vec",
    "Some", "None", "Ok", "Err", "Box", "Vec",
    "matches", "drop", "replace", "take", "swap",
    "if", "while", "for", "match", "return", "let",
    "fn", "pub", "mut", "ref", "const", "static",
    "do", "in", "as",
    "self", "Self",
}

# Single-letter names and very short generic-style identifiers — too noisy.
def _is_noise_name(name: str) -> bool:
    if not name:
        return True
    if len(name) <= 2:
        return True
    if name in STD_METHOD_BLOCKLIST:
        return True
    if name[0].isupper():
        # treat capitalized identifiers as types/variants, not fn-call sites
        # we want fns; rust style: snake_case
        return True
    return False


# Parse Rust source: very rough.
FN_DECL_RE = re.compile(
    r"\bfn\s+([a-z_][a-z0-9_]*)\s*[<(]",
    re.MULTILINE,
)
PUB_FN_RE = re.compile(
    r"\bpub(?:\s*\(\s*[^)]+\s*\))?\s+fn\s+([a-z_][a-z0-9_]*)\s*[<(]",
    re.MULTILINE,
)
CALL_RE = re.compile(r"\b([a-z_][a-z0-9_]*)\s*\(")


def read_rust_sources(crate: Path) -> list[Path]:
    src = crate / "src"
    if not src.exists():
        return []
    return sorted(p for p in src.rglob("*.rs"))


def parse_crate(crate: Path) -> tuple[set[str], dict[str, str]]:
    """Return (entry_pub_fn_names, fn_name -> body_text).

    Body is approximated as everything between the fn declaration and the
    matching closing brace (using a brace counter). Good enough for SUCCESS
    samples whose lib.rs is small.
    """
    entries: set[str] = set()
    bodies: dict[str, str] = {}

    for path in read_rust_sources(crate):
        text = path.read_text()
        # entries: pub fn <name>
        for m in PUB_FN_RE.finditer(text):
            entries.add(m.group(1))
        # all fns + their bodies
        for m in FN_DECL_RE.finditer(text):
            name = m.group(1)
            # find first '{' after match start
            i = text.find("{", m.end())
            if i < 0:
                continue
            depth = 0
            j = i
            while j < len(text):
                c = text[j]
                if c == "{":
                    depth += 1
                elif c == "}":
                    depth -= 1
                    if depth == 0:
                        bodies[name] = text[i + 1 : j]
                        break
                j += 1

    return entries, bodies


def adjacency_closure(
    entries: Iterable[str],
    bodies: dict[str, str],
) -> set[str]:
    """BFS over call edges starting from entries, restricted to fns in `bodies`."""
    closure: set[str] = set()
    work = list(entries)
    while work:
        fn = work.pop()
        if fn in closure:
            continue
        closure.add(fn)
        body = bodies.get(fn, "")
        for m in CALL_RE.finditer(body):
            callee = m.group(1)
            if _is_noise_name(callee):
                continue
            if callee == fn:
                continue
            if callee in bodies and callee not in closure:
                work.append(callee)
    return closure


# ---- product-side search --------------------------------------------------

def collect_product_text(output: Path) -> str:
    exts = (".lean", ".v", ".fst", ".sml")
    chunks = []
    for p in output.rglob("*"):
        if p.is_file() and p.suffix in exts:
            try:
                chunks.append(p.read_text())
            except UnicodeDecodeError:
                continue
    return "\n".join(chunks)


def candidate_patterns(name: str, tool: str) -> list[re.Pattern]:
    """Build a list of mangling-tolerant patterns for a fn name."""
    n = re.escape(name)
    pats = [
        # bare token boundary
        re.compile(rf"(?<![A-Za-z0-9_]){n}(?![A-Za-z0-9_])"),
        # rust-path mangle: ::name / .name / __name / _name
        re.compile(rf"(?:::|\.|__|/){n}(?![A-Za-z0-9_])"),
    ]
    if tool == "rocq-of-rust":
        # rocq emits "lib::<path>::<name>" strings + Definition <name>
        pats.append(re.compile(rf'"lib::[^"]*?\b{n}\b[^"]*"'))
        pats.append(re.compile(rf"\bDefinition\s+{n}\b"))
        pats.append(re.compile(rf"\bModule\s+{n}\b"))
    elif tool.startswith("aeneas"):
        # snake_case definitions; sometimes namespace-prefixed
        pats.append(re.compile(rf"\b(?:def|Definition|let|Define)\s+\S*?\b{n}\b"))
    elif tool == "hax-lean":
        # def <prefix>.<name> ; the prefix is the enclosing fn in hax
        pats.append(re.compile(rf"\bdef\s+\S*?\.{n}\b"))
        pats.append(re.compile(rf"\bdef\s+{n}\b"))
    return pats


def find_hits(name: str, tool: str, blob: str) -> bool:
    for p in candidate_patterns(name, tool):
        if p.search(blob):
            return True
    return False


# ---- main -----------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", required=True, type=Path)
    ap.add_argument("--output", required=True, type=Path)
    ap.add_argument("--tool", required=True)
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    if not args.src.exists():
        print(f"src not found: {args.src}", file=sys.stderr)
        return 2
    if not args.output.exists():
        print(f"output not found: {args.output}", file=sys.stderr)
        return 2

    entries, bodies = parse_crate(args.src)
    if not entries:
        # Some entries (e.g. creusot-limit/mutual-recursion) only have plain
        # `fn` and one `pub fn` driver. Fall back to all fns.
        entries = set(bodies.keys())

    expected = adjacency_closure(entries, bodies)

    blob = collect_product_text(args.output)
    if not blob:
        print(f"warning: no .lean/.v/.fst/.sml files found under {args.output}",
              file=sys.stderr)

    hits, misses = [], []
    for fn in sorted(expected):
        (hits if find_hits(fn, args.tool, blob) else misses).append(fn)

    crate_name = args.src.name
    print(f"crate:        {crate_name}")
    print(f"tool:         {args.tool}")
    print(f"expected fns: {len(expected)} {sorted(expected)}")
    print(f"hit:          {len(hits)} / miss: {len(misses)}")
    print(f"missing:      {misses}")

    return 0 if not misses else 1


if __name__ == "__main__":
    sys.exit(main())
