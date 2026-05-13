# Tutorial - Run Manually and Read Results

Chinese original: [tutorial-cn.md](tutorial-cn.md)

This tutorial is not a generic "how to cargo build" guide. It is a practical, reproducible walkthrough showing how to run real demos with `runner` and how to interpret outputs in three layers:

- `results.json` (machine-readable summary)
- `report.md` (human-readable matrix)
- `raw/<tool>/<entry>.{stdout,stderr,exit}` (ground truth)

After this tutorial, you should be able to:

- run arbitrary subsets of tools and entries;
- explain `SUCCESS` or `FAILED` by evidence in `stderr` and artefacts;
- understand how wrappers/oracles map subprocess outcomes into final statuses.

Related design references:

- [docs/design/principles.md](../design/principles.md)
- [docs/publish/publish-readiness.md](publish-readiness.md)

---

## 0. Preparation

### 0.1 Requirements

- macOS or Linux
- Rust/Cargo installed
- at least one stable toolchain

### 0.2 Clone and Build

```sh
git clone <repo-url>
cd rust-ver-tool-testsuite
git submodule update --init --recursive
cp .env.example .env
cargo build -p runner --release
```

If you only run `cargo-check` demos, `.env` is usually unnecessary.

---

## 1. Demo 1 - Minimal Case (`cargo-check` x 1 entry)

### 1.1 Inspect entry files

```sh
ls examples/hello/basic-hello/
cat examples/hello/basic-hello/hirusttest.toml
cat examples/hello/basic-hello/Cargo.toml
cat examples/hello/basic-hello/src/lib.rs
```

### 1.2 Run one task

```sh
runner --tool cargo-check --entry 'hello/**'
```

### 1.3 Inspect run directory

```sh
ls runs/
ls runs/<run-id>/
```

You should see:

- `report.md`
- `results.json`
- `raw/`

### 1.4 Check raw outputs (ground truth)

```sh
cat runs/<run-id>/raw/cargo-check/<slug>.exit
cat runs/<run-id>/raw/cargo-check/<slug>.stdout
cat runs/<run-id>/raw/cargo-check/<slug>.stderr
```

### 1.5 Check `results.json`

Inspect one result object and verify:

- `tool`, `entry_id`, `status`
- `exit_code`
- `stdout_path` / `stderr_path`
- timestamps and metadata

---

## 2. Demo 2 - Multiple entries x one tool

```sh
runner --tool cargo-check --entry 'closure-adv/**' --entry 'enum/**'
```

Use this to verify filtering and task expansion behaviour.

---

## 3. Demo 3 - A real `FAILED`

Pick a known unsupported shape for a given tool (for example, inline assembly in a frontend mode that rejects it), run that slice, then inspect:

- wrapper diagnostics
- tool stderr
- final `FAILED` status

Key point: do not explain failure by percentage tables alone; always trace the concrete signal in raw outputs.

---

## 4. Demo 4 - Validate entry-focused interpretation

Use a case where warnings may come from transitive dependencies or standard library internals. Verify how oracle logic anchors judgement to the declared measurement scope and documented rules.

---

## 5. How to read `results.json` and `report.md`

### 5.1 `results.json` essentials

Important top-level blocks:

- run metadata (time, host, versions)
- task list (`results[]`)
- per-task paths to raw artefacts

### 5.2 `report.md` essentials

Main sections typically include:

- run metadata
- per-tool summary
- per-feature summary
- entry x tool matrix

---

## 6. Existing snapshots and reports

- Main run artefacts: [runs/](../../runs/)
- Per-tool deep audits: [deep-reports/cc-reports/](../../deep-reports/cc-reports/)

---

## 7. Add a new entry (minimal flow)

1. Create `examples/<feature>/<dir>/Cargo.toml`
2. Create `examples/<feature>/<dir>/src/lib.rs`
3. Create `examples/<feature>/<dir>/hirusttest.toml`
4. Run a focused slice:

```sh
runner --tool cargo-check --entry '<feature>/**'
```

---

## 8. Add a new tool (minimal flow)

1. Add `tools/<name>/tool.toml`
2. Add `tools/<name>/harness.rs.tera`
3. Run one entry:

```sh
runner --tool <name> --entry 'hello/**'
```

4. Verify strict `SUCCESS` signal and partial exposure rules in tool README.

---

## 9. Full-matrix run

```sh
cargo run --release --manifest-path runner/Cargo.toml
```

For large runs:

- keep metadata
- inspect tool versions
- avoid over-interpreting a single timestamped snapshot

---

## 10. Transparency tools

### 10.1 `prepare` (pre-spawn state)

```sh
runner prepare <tool> <feature>/<dir>/<entry-fn>
```

Use this to inspect exactly what the tool will receive.

### 10.2 `--keep-work-dir` (post-spawn state)

```sh
runner --keep-work-dir --tool <tool> --entry '<glob>'
```

Use this to inspect what the tool leaves behind after execution.

---

## 11. FAQ

### 11.1 Missing `TS_*` env var

Set required variables in `.env` according to the target tool README.

### 11.2 Tool root/path errors

Validate local installation paths and wrapper configuration.

### 11.3 Missing fields in `results.json`

Check runner version and whether the run completed normally.

### 11.4 Where wrapper diagnostics appear

Usually in `raw/<tool>/<slug>.stderr`.

---

## 12. Next steps

- Read [tutorial-execution-walkthrough.md](tutorial-execution-walkthrough.md) for stage-by-stage pipeline transparency.
- Read [publish-readiness.md](publish-readiness.md) for external-audit criteria.

