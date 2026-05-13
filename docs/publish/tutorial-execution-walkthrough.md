# Tutorial: Runner Execution Walkthrough - Shape, Guarantees, and Evidence

Chinese original: [tutorial-execution-walkthrough-cn.md](tutorial-execution-walkthrough-cn.md)

This document complements [tutorial.md](tutorial.md):

- `tutorial.md` focuses on how to run experiments and read results.
- This file explains what `runner` produces at each stage, which constitutional rule each stage supports, and how to audit those claims with `prepare` and `--keep-work-dir`.

Intended readers:

- reviewers and reproducer engineers;
- contributors adding new tools;
- anyone verifying that runner behaviour is auditable and non-cheating.

---

## 0. Two transparency windows

Runner normally cleans up `work_dir`. These two interfaces expose intermediate states:

```sh
# stop before spawn: inspect exact tool input shape
runner prepare <tool> <entry-id>

# run fully but keep work dirs: inspect post-spawn artefacts
runner --keep-work-dir [--tool ...] [--entry ...]
```

`<entry-id>` format: `<feature>/<dir>/<entry-fn>`.

---

## 1. Bird's-eye pipeline (9 stages)

1. discover examples and tools
2. filter and expand cartesian task set
3. copy and isolate each task into a work directory
4. patch `Cargo.toml` if `extra_cargo_deps` are declared
5. render harness (bin-mode or lib-mode)
6. build spawn environment (strip/inject env vars)
7. spawn process group and capture outputs
8. persist raw artefacts and optionally clean up work dir
9. classify result and generate reports

---

## 2. Stage 1 - discover

### What it does

Scans signal files under `examples/` and `tools/`.

### Shape produced

- `Vec<Example>`
- `Vec<Tool>`

### Why it matters

Defines the measurable universe without central hardcoded registries.

### How to verify

Run a narrow slice and confirm discovered IDs appear in outputs.

---

## 3. Stage 2 - filter + cartesian expansion

### What it does

Applies `--tool` and `--entry` filters, then forms task triples.

### Shape produced

`Vec<(Tool, Example, Entry)>`

### Why it matters

No silent dropping: requested scope must map to explicit tasks.

### How to verify

Compare expected cardinality vs `Running N task(s)` logs.

---

## 4. Stage 3 - copy + isolate

### What it does

Creates isolated task work dirs under run output.

### Why it matters

Supports non-intrusion: original examples remain untouched.

### How to verify

Diff original example against work dir before render side effects.

---

## 5. Stage 4 - patch `Cargo.toml`

### What it does

Injects `extra_cargo_deps` declared in tool config when needed.

### Why it matters

Heterogeneity stays declarative (`tool.toml`), not hardcoded in runner logic.

### How to verify

Run `prepare` on a tool that requires injection and inspect generated manifest.

---

## 6. Stage 5 - render harness

### What it does

Renders Tera templates using declared variables, then injects measurement entry.

- bin-mode: adds `src/bin/__ts_harness.rs`
- lib-mode: rewrites `src/lib.rs` and preserves original logic in `__ts_inner.rs`

### Why it matters

Implements standardised execution entry while preserving entry-target anchoring.

### How to verify

Inspect rendered files in `prepare` output work dir.

---

## 7. Stage 6 - spawn environment construction

### What it does

Strips framework-only vars and injects task anchors such as:

- `TS_ENTRY_FN`
- `TS_TARGET_CRATE`

plus tool-specific env entries.

### Why it matters

Keeps oracle anchoring explicit and reproducible.

### How to verify

Use verbose logs and confirm env injection for a selected task.

---

## 8. Stage 7 - spawn + capture

### What it does

Runs tool command in a dedicated process group, captures stdout/stderr, applies timeout handling.

### Why it matters

Ensures robust process control and complete capture of evidence.

### How to verify

Inspect raw artefacts and timeout behaviour on long-running tasks.

---

## 9. Stage 8 - cleanup / keep-work-dir

### What it does

Writes raw artefacts, then either removes or preserves work dirs.

### Why it matters

`--keep-work-dir` enables post-run forensic inspection.

### How to verify

Run with and without `--keep-work-dir`; compare directory retention.

---

## 10. Stage 9 - classify + report

### What it does

Maps execution outcomes into status classes and emits:

- `results.json`
- `report.md`

### Why it matters

Classification is the formal bridge between raw evidence and matrix-level interpretation.

### How to verify

Cross-check one task across:

- raw exit/stdout/stderr
- `results.json` task record
- `report.md` matrix cell

---

## 11. Stage summary table

| Stage | Input | Output | Primary guarantee |
|---|---|---|---|
| discover | filesystem | examples/tools lists | explicit measurable scope |
| filter/cartesian | lists + filters | task list | no silent dropping |
| copy/isolate | task | work dir | no mutation of source examples |
| patch manifest | work dir + tool config | patched manifest (if needed) | declarative heterogeneity |
| render harness | work dir + entry | executable harness shape | consistent entry anchoring |
| build env | task metadata | spawn env | explicit oracle anchors |
| spawn/capture | command + env | exit/stdout/stderr | reproducible evidence capture |
| cleanup/retain | work dir | cleaned/kept state | auditable transparency mode |
| classify/report | raw artefacts | statuses + reports | formal interpretation channel |

---

## 12. Anti-cheat self-checklist

1. Original example files remain unchanged on disk.
2. Bin-mode does not mutate original entry source.
3. Lib-mode preserves original logic in `__ts_inner.rs`.
4. Tool boundary flags match documented frontend cut lines.
5. Oracle anchor vars (`TS_ENTRY_FN`, `TS_TARGET_CRATE`) are injected.
6. Raw artefacts reflect actual tool execution, not mocked outcomes.

Use `prepare` and `--keep-work-dir` to verify all six items empirically.

---

## 13. Closing note

This walkthrough is a transparency and audit aid. For user-level operation, start with [tutorial.md](tutorial.md). For publication-grade audit workflow, continue to [publish-readiness.md](publish-readiness.md).

