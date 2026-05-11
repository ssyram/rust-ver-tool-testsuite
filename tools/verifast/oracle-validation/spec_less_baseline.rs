// Oracle-validation micro-test for tools/verifast/verifast-strict-wrapper.sh.
//
// Purpose: confirm that a SPEC-LESS entry produces N ≤ 40 (the prelude
// baseline) so the wrapper REJECTS this case as vacuous-pass (a true漏报 the
// rule must catch).
//
// Companion to: tools/verifast/oracle-validation/spec_bearing_add_one.rs
// Documented in: docs/fixes/oracle-leak-rules-implementation-2026-05-08.md §2.1
//
// This file is OUTSIDE the corpus (no hirusttest.toml here) — it is run by
// hand to validate the rule, never by the matrix runner.

#![no_std]

/// No `//@ req/ens` — under -skip_specless_fns, verifast skips this fn
/// entirely. stdout's "N statements verified" reflects only the prelude.
pub fn add_one(x: i32) -> i32 {
    x + 1
}
