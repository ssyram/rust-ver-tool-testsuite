use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::host::HostInfo;

#[derive(Serialize, Deserialize, Clone)]
pub struct ToolMeta {
    pub name: String,
    /// Full argv used by the runner.
    pub command: Vec<String>,
    pub timeout_secs: u64,
    pub entry_mode: String,
    pub extra_cargo_deps: Vec<String>,
    /// Captured by running `version_command` once at run start. None means the
    /// tool didn't declare `version_command` or capture failed.
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RunMeta {
    pub run_id: String,
    pub run_started_at: String,    // ISO 8601 UTC
    pub run_finished_at: String,   // ISO 8601 UTC
    pub run_started_unix_secs: u64,
    pub run_finished_unix_secs: u64,
    pub host: HostInfo,
    /// Actual rayon parallelism used in this run.
    pub parallelism: usize,
    /// Per-tool metadata + captured version string.
    pub tools: Vec<ToolMeta>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskResult {
    /// Full ID: "<feature>/<dir>/<entry-fn>"
    pub entry_id: String,
    pub tool: String,
    /// One of: "SUCCESS" (subprocess exit 0), "FAILED" (subprocess ran and
    /// returned non-zero, or was killed by timeout), "UNKNOWN" (runner-internal
    /// error before/around subprocess — does not reflect on the tool's ability
    /// to handle the example).
    pub status: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    /// True if the task was killed because it exceeded `tool.timeout_secs`.
    /// Always implies `status == "FAILED"`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub timed_out: bool,
    /// Relative path (from run_dir) to raw stdout; None for UNKNOWN tasks.
    pub raw_stdout: Option<String>,
    pub raw_stderr: Option<String>,
    /// Present only for UNKNOWN: the runner-internal error string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn is_false(b: &bool) -> bool { !*b }

/// Classify FAILED stderr/stdout against a fixed catalogue of *external* root
/// causes — situations where the tool's own front-end never had a chance to
/// reject the entry's Rust features, so reporting FAILED would violate the
/// "0 误报" principle (principles.md §六 + false-positive-audit-2026-05-11.md
/// §5). When a pattern hits, the task is re-classified as `UNKNOWN` and the
/// returned tag is preserved in `TaskResult::error` for diagnosability.
///
/// Patterns are intentionally specific to avoid swallowing real tool partial
/// signals (e.g., we require both `error[E0061]` *and* the "argument(s)"
/// follow-up so an unrelated arity error in user source can't be misread).
/// `None` means "no external fault recognized — keep the original FAILED".
pub fn classify_external_fault(stderr: &str, stdout: &str, _exit: Option<i32>) -> Option<&'static str> {
    // Some tools (notably kani) route their captured cargo build diagnostics
    // through stdout rather than stderr; check both streams so the rules are
    // resilient to tool-wrapper choices. We never inspect content the runner
    // produced itself — only the child process's captured output.
    let contains_either = |needle: &str| stderr.contains(needle) || stdout.contains(needle);

    // 1. runnable-corpus harness/argument mismatch fallback.
    //    Task Y already fixes the dominant case at render time, but if a
    //    template ever drifts back to zero-arg or a future runnable corpus
    //    ships without a `[runnable.<entry>]` table, we still want to mark
    //    that as runner-fault UNKNOWN rather than blame the tool.
    if contains_either("error[E0061]") && contains_either("argument") {
        return Some("runnable_harness_arg_mismatch");
    }
    // 2. tool pipeline doesn't run `cargo build` (single-file pipelines like
    //    verus / verifast / soteria / rocq-of-rust) → external crates from
    //    `extra_cargo_deps` show up as unresolved imports. The entry itself
    //    is legal Rust (cargo-check passes); the tool just never saw the
    //    deps. Audit §4.2 — 101 FPs.
    //
    //    R3 (2026-05-11): extend to also capture E0433 ("failed to resolve" /
    //    "undeclared" / "cannot find module or crate"). Both E0432 and E0433
    //    are dependency-resolution failures triggered when the tool's single-
    //    file pipeline skips vendored crates declared via `extra_cargo_deps`.
    //    cargo-check SUCCESS on the same entry is the calibration premise
    //    (rules out user path bugs). See
    //    docs/fixes/audit-v5-cc-counter-challenge-2026-05-11.md §3.1.2.
    if contains_either("error[E0432]: unresolved import") {
        return Some("dependency_resolution");
    }
    if contains_either("error[E0433]")
        && (contains_either("failed to resolve")
            || contains_either("undeclared")
            || contains_either("cannot find module or crate"))
    {
        return Some("dependency_resolution");
    }
    // 3. tool ships an old cargo (e.g. prusti 0.1.0 → cargo from 2023-08)
    //    that doesn't know about Rust 2024 edition. Audit §4.3 — 7 FPs.
    if contains_either("this version of Cargo is older than the")
        && contains_either("edition")
    {
        return Some("toolchain_edition_mismatch");
    }
    // 4. vendor crate lint level + tool-specific toolchain combination. The
    //    `vendor/x509-parser` crate declares `#![deny(unused_qualifications)]`
    //    and newer rustc fires the lint more aggressively. Audit §4.4 — 10
    //    FPs (kani + hax-*). Match on the lint name *and* a vendored path so
    //    we don't catch a user entry that legitimately writes
    //    `unused_qualifications` somewhere.
    if contains_either("unused_qualifications") && contains_either("vendor/") {
        return Some("vendor_lint_strictness");
    }
    // 5. environment corruption — historical example: prusti viper_tools jars
    //    cleaned out of /tmp, leading to JNI `JavaException` on the unwrap of
    //    Result. Generic enough to cover other JVM-class crashes that bubble
    //    up the same way without misclassifying a real prusti partial
    //    (prusti's own partial signals are tagged `[Prusti: unsupported …]`
    //    or `[Prusti: internal error]`, never `JavaException`).
    if contains_either("Result::unwrap()") && contains_either("JavaException") {
        return Some("environment_corruption");
    }
    // 6. rustc edition pipeline propagation. Tools that drive rustc directly
    //    (verus / verifast / soteria / rocq-of-rust*) without forwarding
    //    `--edition` from the entry's Cargo.toml end up running under the
    //    default Rust 2015 edition. Entries that legitimately declare
    //    `edition = "2021"` or `"2024"` then trip rustc's edition gate (e.g.
    //    E0670 for `async fn`, or "let chains are only allowed in Rust 2024
    //    or later"). cargo-check on the same entry is SUCCESS, so the entry
    //    is legal Rust — the failure is purely the tool's pipeline dropping
    //    the edition flag. Two-signal pattern: an edition-gated rustc error
    //    keyword (E0670 / "let chains") AND an edition-version phrase
    //    ("Rust 2015" / "Rust 2021" / "Rust 2024" / "only allowed"). See
    //    docs/fixes/audit-v5-cc-counter-challenge-2026-05-11.md §3.1.1.
    if (contains_either("E0670") || contains_either("let chains"))
        && (contains_either("Rust 2015")
            || contains_either("Rust 2021")
            || contains_either("Rust 2024")
            || contains_either("only allowed"))
    {
        return Some("edition_pipeline_propagation");
    }
    None
}

/// Owned shape used both when serialising results.json and when reading it
/// back via the `runner report <run-dir>` subcommand.
#[derive(Serialize, Deserialize)]
pub struct ResultsFile {
    #[serde(flatten)]
    pub meta: RunMeta,
    pub results: Vec<TaskResult>,
}

/// Write run_dir/results.json — machine-readable summary including run-level
/// metadata (host info, timestamps, per-tool versions) so the raw data is
/// fully self-describing without external context.
pub fn write_results_json(
    run_dir: &Path,
    meta: &RunMeta,
    results: &[TaskResult],
) -> Result<()> {
    // `ResultsFile` owns its data; clone refs into owned for one-off serialise.
    let file = ResultsFile {
        meta: meta.clone(),
        results: results.to_vec(),
    };
    let json = serde_json::to_string_pretty(&file).context("serializing results.json")?;
    let path = run_dir.join("results.json");
    std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Write run_dir/report.md — human-readable matrix grouped by feature, with a
/// metadata header (run time, host, per-tool versions) at top.
pub fn write_report_md(run_dir: &Path, meta: &RunMeta, results: &[TaskResult]) -> Result<()> {
    // Group: feature → entry_local_id ("dir/entry-fn") → tool → result
    let mut groups: BTreeMap<String, BTreeMap<String, BTreeMap<String, &TaskResult>>> =
        BTreeMap::new();
    let mut all_tools: BTreeMap<String, ()> = BTreeMap::new();

    for r in results {
        let mut parts = r.entry_id.splitn(2, '/');
        let feature = parts.next().unwrap_or("").to_string();
        let local = parts.next().unwrap_or("").to_string();
        groups
            .entry(feature)
            .or_default()
            .entry(local)
            .or_default()
            .insert(r.tool.clone(), r);
        all_tools.insert(r.tool.clone(), ());
    }

    let tools: Vec<&str> = all_tools.keys().map(String::as_str).collect();

    let mut md = String::new();
    md.push_str(&format!("# Run {}\n\n", meta.run_id));

    // Metadata block — always render so the raw matrix below is self-describing.
    md.push_str("## Run metadata\n\n");
    md.push_str(&format!("- **started**:  `{}`\n", meta.run_started_at));
    md.push_str(&format!("- **finished**: `{}`\n", meta.run_finished_at));
    let secs = meta
        .run_finished_unix_secs
        .saturating_sub(meta.run_started_unix_secs);
    md.push_str(&format!("- **duration**: {} s wall\n", secs));
    md.push_str(&format!(
        "- **host**:     `{}` ({} / {} {}, {})\n",
        meta.host.hostname.as_deref().unwrap_or("?"),
        meta.host.os,
        meta.host.arch,
        meta.host.kernel.as_deref().unwrap_or(""),
        meta.host.cpu_brand.as_deref().unwrap_or("?cpu"),
    ));
    md.push_str(&format!(
        "- **memory / cores**: {} MB / {} cpus (parallelism = {})\n\n",
        meta.host.total_mem_mb.unwrap_or(0),
        meta.host.num_cpus.unwrap_or(0),
        meta.parallelism,
    ));
    md.push_str("### Tool versions\n\n");
    md.push_str("| tool | version |\n|---|---|\n");
    for t in &meta.tools {
        let v = t
            .version
            .as_deref()
            .unwrap_or("(no version_command configured)");
        // First line of the captured version string for the table row;
        // keep markdown table single-line so it renders cleanly. Replace
        // newlines with " · " separator.
        let one_line = v.replace('\n', " · ");
        md.push_str(&format!("| {} | {} |\n", t.name, one_line));
    }
    md.push('\n');

    // ── Aggregate tables ────────────────────────────────────────────────────
    // Per-tool: S / F / U / TIMEOUT counts + rate, plus duration distribution
    // (avg / p50 / p90 / max in milliseconds). Sorted by SUCCESS rate desc.
    md.push_str("## Per-tool summary\n\n");
    md.push_str("Sorted by SUCCESS rate. Duration columns are ms (avg / median / p90 / max). Time fields are environmental — not a tool-quality score.\n\n");
    md.push_str("| tool | n | S | F | U | TO | rate | avg | p50 | p90 | max |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    let mut by_tool: BTreeMap<&str, Vec<&TaskResult>> = BTreeMap::new();
    for r in results {
        by_tool.entry(r.tool.as_str()).or_default().push(r);
    }
    let mut tool_rows: Vec<(u32, String, String)> = Vec::new();
    for (tool, rs) in &by_tool {
        let n = rs.len();
        if n == 0 {
            continue;
        }
        let s = rs.iter().filter(|r| r.status == "SUCCESS").count();
        let f = rs.iter().filter(|r| r.status == "FAILED").count();
        let u = rs.iter().filter(|r| r.status == "UNKNOWN").count();
        let to = rs.iter().filter(|r| r.timed_out).count();
        let rate_pct = (s * 100) / n;
        let mut durs: Vec<u128> = rs.iter().map(|r| r.duration_ms).collect();
        durs.sort_unstable();
        let avg = durs.iter().copied().sum::<u128>() / (n as u128);
        let p50 = durs[n / 2];
        let p90 = durs[(n * 9) / 10];
        let max = *durs.last().unwrap_or(&0);
        let row = format!(
            "| {} | {} | {} | {} | {} | {} | {}% | {} | {} | {} | {} |\n",
            tool, n, s, f, u, to, rate_pct, avg, p50, p90, max,
        );
        tool_rows.push((rate_pct as u32, tool.to_string(), row));
    }
    tool_rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (_, _, row) in &tool_rows {
        md.push_str(row);
    }
    md.push('\n');

    // Per-feature: tasks (= entries × tools that ran here), S / F / rate.
    md.push_str("## Per-feature summary\n\n");
    md.push_str("Tasks = entries in this feature × tools that ran on them. Sorted by SUCCESS rate.\n\n");
    md.push_str("| feature | tasks | S | F | rate |\n|---|---:|---:|---:|---:|\n");
    let mut by_feat: BTreeMap<String, Vec<&TaskResult>> = BTreeMap::new();
    for r in results {
        let feat = r.entry_id.split('/').next().unwrap_or("").to_string();
        by_feat.entry(feat).or_default().push(r);
    }
    let mut feat_rows: Vec<(u32, String, String)> = Vec::new();
    for (feat, rs) in &by_feat {
        let n = rs.len();
        if n == 0 {
            continue;
        }
        let s = rs.iter().filter(|r| r.status == "SUCCESS").count();
        let f = rs.iter().filter(|r| r.status == "FAILED").count();
        let rate_pct = (s * 100) / n;
        let row = format!(
            "| {} | {} | {} | {} | {}% |\n",
            feat, n, s, f, rate_pct,
        );
        feat_rows.push((rate_pct as u32, feat.clone(), row));
    }
    feat_rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (_, _, row) in &feat_rows {
        md.push_str(row);
    }
    md.push('\n');

    md.push_str("## Per-feature × per-tool (entry × tool matrix follows below)\n\n");

    for (feature, entries) in &groups {
        md.push_str(&format!("## {}\n\n", feature));

        // Header
        md.push_str("| entry");
        for t in &tools {
            md.push_str(&format!(" | {}", t));
        }
        md.push_str(" |\n");

        // Separator
        md.push_str("|---");
        for _ in &tools {
            md.push_str("|---");
        }
        md.push_str("|\n");

        // Rows
        for (local, by_tool) in entries {
            md.push_str(&format!("| {}", local));
            for t in &tools {
                let cell = match by_tool.get(*t) {
                    Some(r) if r.status == "SUCCESS" => "SUCCESS".to_string(),
                    Some(r) if r.status == "UNKNOWN" => "UNKNOWN".to_string(),
                    Some(r) if r.timed_out => "FAILED (timeout)".to_string(),
                    Some(r) => {
                        let code = r
                            .exit_code
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "-".to_string());
                        format!("FAILED ({})", code)
                    }
                    None => "-".to_string(),
                };
                md.push_str(&format!(" | {}", cell));
            }
            md.push_str(" |\n");
        }
        md.push('\n');
    }

    let s = results.iter().filter(|r| r.status == "SUCCESS").count();
    let f = results.iter().filter(|r| r.status == "FAILED").count();
    let u = results.iter().filter(|r| r.status == "UNKNOWN").count();
    md.push_str(&format!(
        "---\nTotal: {} succeeded / {} failed / {} unknown / {} total\n",
        s, f, u, results.len()
    ));

    let path = run_dir.join("report.md");
    std::fs::write(&path, md).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}
