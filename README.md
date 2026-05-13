# rust-ver-tool-testsuite

Language: English | [简体中文](README-cn.md)

Documentation language policy: this `README.md` is the primary English-facing entry. Most internal project documents are authored in Chinese by default unless a dedicated English version is provided.

Tutorial index (bilingual):

- Manual run and result reading: [EN](docs/publish/tutorial.md) | [中文](docs/publish/tutorial-cn.md)
- Runner execution walkthrough: [EN](docs/publish/tutorial-execution-walkthrough.md) | [中文](docs/publish/tutorial-execution-walkthrough-cn.md)

A neutral, broad-coverage measurement framework for **feature coverage breadth** across Rust verification/analysis tools.

There is a long-standing practical issue in the Rust verification ecosystem: many tools, many terms, and many incompatible narratives.
For non-specialists (especially industry users), the hardest question is often not "how to use one tool", but "which tool is actually usable for the Rust features in my codebase". People also care about practical coverage ceilings and what level of feature support each tool can realistically reach.

This project does two things:

- Provides a neutral execution framework ([runner/](runner/)): run many tools over the same corpus with the same workflow.
- Provides a reusable example corpus ([examples/](examples/)): Rust-feature-organised examples with standard zero-argument `pub fn` entry points.

**Statement**: this project provides a neutral measurement framework and corpus. The integrated tool runs are provided as demonstrative examples to show that the framework operates correctly and can be broadly used for measurement runs across concrete Rust tools. We do not provide long-term guarantees for any specific tool. Any reported coverage result is valid only for its corresponding time, version set, and environment, and must not be treated as a final judgement or quality ranking.

**License**: Dual-licensed under Apache-2.0 OR MIT. See [LICENSE](LICENSE).
Vendored crates under [vendor/](vendor/) retain their upstream licences.

---

## Quick Start

```sh
git clone <repo>
cd rust-ver-tool-testsuite
git submodule update --init --recursive   # Required: parts of industrial/ depend on vendor/rsa /sha2 /x509-parser
cp .env.example .env                       # Edit paths for your local machine
cargo run --release --manifest-path runner/Cargo.toml
```

Outputs are written to `runs/run-<unix-ts>-<pid>/` (`report.md` + `results.json` + `raw/`).

---

## Sample Run Results

### Boundaries and Non-Guarantees (Read First)

- We measure **frontend support boundaries**: whether a tool can fully accept an example within its own frontend boundary.
- We do not measure backend solving power, proof validity, or translation semantic fidelity.
- We use strict oracles: no partial or silent-skip is allowed; if a tool self-reports "not fully done", it is `FAILED`.
- What we guarantee is **reproducibility of the measurement procedure under this rule set**, not long-term guarantees of tool capability.

Results are valid only for a specific run and must be interpreted together with timestamp, tool versions, and host environment.

---

### Pre-Provided Reference Results (With Spatiotemporal Anchors)

The following figures are currently provided as reference observations in this repository (triple-run strict-oracle consolidated view):

- Time anchor: 2026-05-11 (merged from v1+v2+v3)
- Environment anchor: Apple M5 / macOS aarch64 / 24 GB / 10 CPUs
- Scale anchor: 19 tools × 146 entries = 2774 tasks

| Tool | Coverage (reference) |
|---|---:|
| cargo-check | 100.0% |
| miri | 97.3% |
| charon-poly | 95.2% |
| charon-mono | 94.5% |
| kani | 93.2% |
| hax-fstar | 77.4% |
| rocq-of-rust | 76.0% |
| hax-lean | 75.3% |
| creusot | 72.6% |
| soteria | 74.7% |
| hax-coq | 65.8% |
| aeneas-coq / fstar / lean | 59.6% |
| prusti | 38.4% |
| verus | 34.9% |
| aeneas-hol4 | 34.9% |
| kmir | 31.5% |
| verifast | 8.2% |

These figures are intended as an initial reference for tool selection and do not constitute a long-term guarantee. For exact per-tool configuration, oracle rules, and known blind spots, see each tool README and [deep-reports/](deep-reports/).

---

## Documentation Structure (Developer Must-Read; This Is the Source of Truth)

Everything is centred on [docs/design/](docs/design/). For any change, update documentation first, then code/config/measurements. If documentation conflicts with downstream artefacts, fix downstream artefacts rather than docs (unless an upstream revision is explicitly discussed).

```text
docs/design/
├── principles.md      <- "Constitution": absolute spirit; do not revise without explicit discussion; downstream must fully comply
├── architecture.md    <- core architecture under constitutional constraints; index entry
└── detailed-design.md <- function-level details, schemas, config examples
docs/publish/
├── tutorial.md           <- English tutorial: manual run + result reading
├── tutorial-cn.md        <- Chinese original of tutorial
├── tutorial-execution-walkthrough.md    <- English execution walkthrough
├── tutorial-execution-walkthrough-cn.md <- Chinese original of execution walkthrough
├── publish-readiness.md  <- publication audit methodology + current readiness checklist + re-audit triggers (QA L4)
├── paper.md              <- ISSTA-style paper draft (Chinese)
├── glossary.md           <- project-specific terms mapped to academic register
└── tool-citations.md     <- citations for 20 upstream tools
```

**Four QA layers** (see [docs/publish/publish-readiness.md](docs/publish/publish-readiness.md) §7):

- L1 Constitution / Architecture / Implementation ([docs/design/](docs/design/) + [runner/](runner/) + [tools/](tools/))
- L2 Per-tool empirical audits ([deep-reports/cc-reports/](deep-reports/cc-reports/))
- L3 Legal-layer audits ([docs/audit/v6-law-*](docs/audit/))
- L4 Publish-readiness audit ([docs/publish/publish-readiness.md](docs/publish/publish-readiness.md))

Every layer follows Constitution §8 c+cc disprove-first protocol.

[docs/design/principles.md](docs/design/principles.md) is the project constitution.
[docs/design/architecture.md](docs/design/architecture.md) is the core architecture under constitutional constraints.

The sections below (how to run / add examples / add tools) are user-facing application guidance and do not replace constitutional documents.

---

## How to Run

### Full Matrix

```sh
cargo run --release --manifest-path runner/Cargo.toml
```

Without parameters, this runs the full `examples/` × `tools/` matrix. All `(tool, example, entry)` triples are executed concurrently.

### Subset Filtering

```sh
# Run only selected tools
runner --tool kani --tool charon-poly

# Run only entries matching globs
runner --entry 'closure-adv/**' --entry 'enum/**'

# Combined filtering (incremental debugging)
runner --tool prusti --entry '**/btreemap_basic'
```

### Regenerate `report.md` (Without Re-running Tools)

```sh
runner report <runs/run-id>/
```

Reads existing `results.json` and regenerates `report.md`. Useful when schema evolves.

### CLI Options

| flag | default | meaning |
|---|---|---|
| `--examples <dir>` | `examples` | examples root |
| `--tools <dir>` | `tools` | tool config root |
| `--runs <dir>` | `runs` | output root |
| `--parallel <N>` | CPU core count | max concurrency |
| `--tool <NAME>` | (empty = all) | run specific tool(s), repeatable |
| `--entry <GLOB>` | (empty = all) | run matching entry IDs, repeatable |
| `--keep-work-dir` | off | keep `work/<exec_id>/` after execution for inspection (rendered harness / patched Cargo.toml / lib-mode `lib.rs` ↔ `__ts_inner.rs`) |

### Subcommands

| command | purpose |
|---|---|
| `runner report <run-dir>` | regenerate `report.md` from `results.json` (no tool re-run) |
| `runner prepare <tool> <entry>` | perform only copy + injection, **without spawning tool**; `entry` must be full ID `<feature>/<dir>/<entry-fn>` |

---

## Reading Results

Each run creates `runs/run-<unix-ts>-<pid>/`:

```text
runs/run-1778060885-78869/
├── report.md            # metadata + feature-grouped tool × entry matrix
├── results.json         # machine-readable (host info / tool versions / timestamps + per-task details)
└── raw/<tool>/
    ├── <feature>__<dir>__<entry>.stdout
    ├── <feature>__<dir>__<entry>.stderr
    └── <feature>__<dir>__<entry>.exit
```

To inspect a failed unit:

```sh
cat raw/<tool>/<feature>__<dir>__<entry>.stderr
```

Top-level `results.json` metadata includes host stamps (hostname / cpu_brand / mem / num_cpus), ISO 8601 start/end timestamps, and each tool's `version_command` output.

### Result Classes

- `SUCCESS` — child process completed with exit code 0
- `FAILED` — child process completed with non-zero exit, killed by signal, or timed out (SIGKILL)
- `UNKNOWN` — runner-side failure (copy / render / spawn), **not attributed to tool capability**

---

## Add a New Example

Minimal three-file layout: `examples/<feature>/<dir>/{Cargo.toml, src/lib.rs, hirusttest.toml}`.

```toml
# examples/myfeat/basic/Cargo.toml
[package]
name = "myfeat_basic"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
```

```rust
// examples/myfeat/basic/src/lib.rs
pub fn myfeat_basic_entry() {
    let _ = 1 + 1;   // plain Rust, no verifier-specific markers
}
```

```toml
# examples/myfeat/basic/hirusttest.toml
entries = ["myfeat_basic_entry"]
```

Constraints:

- Entry must be zero-argument `pub fn` (any return type allowed).
- One entry = one standalone library crate (cross-entry shared helpers are forbidden).
- Source must be plain Rust with zero verifier-specific markers.
- `hirusttest.toml` may specify `target_path = "<rel>"` for multi-crate examples.
- **Formal requirement**: adding `hirusttest.toml` must leave example cargo behaviour byte-identical.

See Constitution §4A in [docs/design/principles.md](docs/design/principles.md).

### Add an External Composite Project (e.g. vendor/x509-parser)

External projects must use directory mode: `<example_dir>/.hirusttest/config.toml`.

```text
vendor/x509-parser/
├── Cargo.toml
├── src/
└── .hirusttest/
    ├── config.toml
    └── <optional helper files>
```

Hard boundaries:

- Simple single-feature examples under `examples/<feature>/<dir>/` must not be upgraded to directory mode.
- External composite projects must use directory mode.
- `hirusttest.toml` and `.hirusttest/` must not co-exist in one directory.
- Adding `.hirusttest/` must preserve byte-identical `cargo build/check/run/test` behaviour.

---

## Add a New Tool

### Mandatory Criteria

New tool integration must satisfy core spirit: configure `tool.toml` so the tool fully runs its own frontend unit (up to, but not into, its backend prover/solver), with no partial or silent skip accepted.

1. Clearly define frontend/backend boundary for that tool's internal pipeline.
2. `tool.toml` command must stop exactly at frontend boundary.
3. `SUCCESS` signal must be strict (exit + artefact/log conditions as needed).
4. Tool README must document:
   - pipeline stage map;
   - frontend/backend cut line;
   - boundary-enforcing flags;
   - `SUCCESS` signal, partial exposure mechanism, and known blind spots.

Anti-cheat implication: dry-run flags must truly feed sample code into the tool's own frontend, not bypass via stock rustc.

### Minimal File Layout

`tools/<name>/{tool.toml, harness.rs.tera}`.

```toml
# tools/mytool/tool.toml
command          = ["cargo", "mytool", "--bin", "__ts_harness"]
timeout_secs     = 300
extra_cargo_deps = ['mytool-helper = "1.0"']
entry_mode       = "lib"
version_command  = ["cargo", "mytool", "--version"]
```

```rust
// tools/mytool/harness.rs.tera
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

Template variables are exactly `{{ target_crate_name }}` and `{{ entry_fn }}`.

---

## Constitutional Spirit (Must-Read)

All governing principles are in [docs/design/principles.md](docs/design/principles.md). All architecture, implementation, and configuration must fully comply.

- Project goal: build a **feature-coverage test framework**.
- Core modules (long-term commitment): [runner/](runner/) + [examples/](examples/).
- Secondary modules (application showcase): [tools/](tools/) + reports.
- Core scope: frontend acceptance boundaries under idealised backend assumption.
- Non-scope: backend solver power, proof difficulty, semantic fidelity of translated artefacts.

Key honesty constraints:

- Formal metrics are the final interpretation channel.
- Upper-bound guarantee first: avoid false positives.
- Known blind spots must be documented.
- No partial acceptance.

Reports are time/version anchored and do not constitute long-term commitments about tool capabilities.

---

## Runtime Environment and Platform

Current baseline test environment:

- Host: Apple M5 / macOS aarch64 (darwin 25.4.0)
- Some tool configurations include macOS arm64/x86_64-specific paths or target triples

Infrastructure principle: runner is platform-agnostic Rust; platform-specific adaptations are tool-local (`tool.toml` / wrappers).

User configuration: all `TS_*` environment variables are listed in [.env.example](.env.example).

---

## Tool Categories and Frontend Boundaries

19 tools are grouped into five internal-pipeline categories. This test suite measures only up to each tool's frontend boundary.

| Category | Frontend measured | Backend excluded | Tools |
|---|---|---|---|
| Compile baseline | rustc type/borrow checks | N/A | cargo-check |
| Interpreters | full MIR / K MIR execution | no SMT backend | miri, kmir |
| Symbolic execution | full symbolic run | SMT solving (Z3) | soteria, verifast |
| Pure translators | output target-language files | downstream prover/checker | charon-mono/poly, creusot, hax×3, aeneas×4, rocq-of-rust |
| Encoding + SMT-backend tools | output backend input files | SMT solving (CBMC/Silicon/Z3) | kani, prusti, verus |

Unified rules:

- Every tool must complete its own designed work unit.
- No partial acceptance.
- No depth weighting between shallow/deep translation.
- Anti-cheat applies.
- No pre-assumed semantic fidelity of translated artefacts.

---

## Current Tool Set (19)

Each row links to the tool-specific README (install, configuration, and limit sub-tests).

| Tool | Category | README |
|---|---|---|
| cargo-check | baseline cargo compile | [tools/cargo-check/README.md](tools/cargo-check/README.md) |
| kani | model checker (frontend `--only-codegen`) | [tools/kani/README.md](tools/kani/README.md) |
| miri | Rust interpreter + UB detector | [tools/miri/README.md](tools/miri/README.md) |
| charon-poly | Rust -> LLBC translator (polymorphic) | [tools/charon-poly/README.md](tools/charon-poly/README.md) |
| charon-mono | Rust -> LLBC translator (monomorphised) | [tools/charon-mono/README.md](tools/charon-mono/README.md) |
| prusti | spec verifier (Viper) | [tools/prusti/README.md](tools/prusti/README.md) |
| creusot | spec verifier (Why3) | [tools/creusot/README.md](tools/creusot/README.md) |
| hax-lean | Rust -> Lean 4 | [tools/hax-lean/README.md](tools/hax-lean/README.md) |
| hax-fstar | Rust -> F* | [tools/hax-fstar/README.md](tools/hax-fstar/README.md) |
| hax-coq | Rust -> Coq/Rocq | [tools/hax-coq/README.md](tools/hax-coq/README.md) |
| aeneas-lean | Rust -> Lean 4 via charon LLBC | [tools/aeneas-lean/README.md](tools/aeneas-lean/README.md) |
| aeneas-fstar | Rust -> F* | [tools/aeneas-fstar/README.md](tools/aeneas-fstar/README.md) |
| aeneas-coq | Rust -> Coq/Rocq | [tools/aeneas-coq/README.md](tools/aeneas-coq/README.md) |
| aeneas-hol4 | Rust -> HOL4 SML | [tools/aeneas-hol4/README.md](tools/aeneas-hol4/README.md) |
| verifast | spec verifier | [tools/verifast/README.md](tools/verifast/README.md) |
| verus | SMT spec verifier | [tools/verus/README.md](tools/verus/README.md) |
| rocq-of-rust | Rust -> Rocq (direct THIR translation) | [tools/rocq-of-rust/README.md](tools/rocq-of-rust/README.md) |
| soteria | Tree Borrows symbolic execution | [tools/soteria/README.md](tools/soteria/README.md) |
| kmir | K Framework MIR operational semantics | [tools/kmir/README.md](tools/kmir/README.md) |

Each tool has matching limit sub-tests (e.g. `examples/<tool>-limit/`) as intentional non-support signals.

---

## Troubleshooting

### Straggler Verifier Processes (cbmc / kompile / etc.)

If runner is interrupted and descendant processes remain:

```sh
./scripts/kill-stragglers.sh
```

This script walks process trees and applies process-group and PID kill cascades.

### Tool Versions and Platforms

Current install instructions assume macOS Apple Silicon (Darwin / aarch64). Linux paths are not yet validated.

Tool versions are pinned in tool READMEs; each run records concrete versions in `results.json` metadata.

---

## Design Notes (Quick View)

See [docs/design/architecture.md](docs/design/architecture.md).

1. Three principles: non-intrusion, necessary-condition measurement, heterogeneity in configuration.
2. Principle A protects original on-disk example source from direct edits.
3. Frontend-support observation: acceptance range, not proof-completion range.
4. Tool non-static principle: every observation must be timestamped and version-anchored.
5. Runtime projection: atomic unit is one entry.

---

## Known Constraints

- Examples and tools are discovered by directory scanning (no central registry).
- Every task runs in a fully isolated copy (`cp -r`, excluding `target/` and `Cargo.lock`).
- Examples are excluded from root workspace checks via root `Cargo.toml` exclude rules.
- Child processes run in independent process groups; timeout kills full process groups.
