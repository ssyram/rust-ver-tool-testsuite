// Oracle-validation micro-test for tools/verifast/verifast-strict-wrapper.sh.
//
// Purpose: confirm that a SPEC-BEARING entry produces N > 40 (so the wrapper's
// vacuous-pass rule does NOT reject it as a 误报).
//
// Companion to: tools/verifast/oracle-validation/spec_less_baseline.rs
// Documented in: docs/fixes/oracle-leak-rules-implementation-2026-05-08.md §2.1
//
// This file is OUTSIDE the corpus (no hirusttest.toml here) — it is run by
// hand to validate the rule, never by the matrix runner.

#![no_std]

/// Minimal spec-bearing fn. `//@ req` makes verifast NOT skip this fn under
/// -skip_specless_fns; the body discharges `//@ ens result == 42`. Spec form
/// is the same as verifast-26.01/tests/rust/preprocessor_test_crlf_bom.rs
/// minus the assume (which would require -allow_assume).
fn foo() -> i32
//@ req true;
//@ ens result == 42;
{
    42
}
