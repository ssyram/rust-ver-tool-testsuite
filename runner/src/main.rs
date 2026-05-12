use anyhow::{Context, Result};
use clap::Parser;
use glob::Pattern;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod discover;
mod exec;
mod host;
mod report;

#[derive(Parser, Debug)]
#[command(name = "runner")]
#[command(about = "Rust verification tool feature-coverage runner")]
struct Args {
    #[command(subcommand)]
    cmd: Option<Cmd>,

    /// Path to examples directory
    #[arg(long, default_value = "examples", global = true)]
    examples: PathBuf,

    /// Path to tools directory
    #[arg(long, default_value = "tools", global = true)]
    tools: PathBuf,

    /// Path to runs output directory
    #[arg(long, default_value = "runs", global = true)]
    runs: PathBuf,

    /// Max parallel executions (default: number of CPU cores)
    #[arg(long, global = true)]
    parallel: Option<usize>,

    /// Only run these tools (by tools/<name> dir name). Repeatable. Empty = all.
    #[arg(long = "tool", value_name = "NAME", global = true)]
    tools_filter: Vec<String>,

    /// Only run entries whose ID `<feature>/<dir>/<entry-fn>` matches this glob.
    /// Repeatable; entry passes if it matches any pattern. Empty = all.
    #[arg(long = "entry", value_name = "GLOB", global = true)]
    entries_filter: Vec<String>,

    /// Skip the final cleanup of `runs/<id>/work/<exec_id>/` — keeps the
    /// isolated working copy of each (tool, entry) task on disk so the
    /// user can inspect what the runner prepared (rendered harness,
    /// patched Cargo.toml, lib-mode lib.rs → __ts_inner.rs rename, etc.).
    /// Default behaviour is to remove the work_dir post-spawn to save disk.
    #[arg(long, global = true)]
    keep_work_dir: bool,
}

#[derive(clap::Subcommand, Debug)]
enum Cmd {
    /// Regenerate report.md from an existing run's results.json (no re-run).
    Report {
        /// Path to a `runs/run-<id>/` directory containing `results.json`.
        run_dir: PathBuf,
    },
    /// Prepare a single (tool, entry) work directory without spawning the
    /// tool subprocess. Outputs the path to the prepared directory so the
    /// user can `cd` into it and inspect the framework's intermediate
    /// state — renamed `src/lib.rs` → `src/__ts_inner.rs` (in lib mode),
    /// rendered harness template, patched Cargo.toml with extra_cargo_deps,
    /// and exact `${TS_*}` env vars that would be applied at spawn.
    Prepare {
        /// Tool name (matches a `tools/<name>/` directory).
        tool: String,
        /// Entry id of the form `<feature>/<dir>/<entry-fn>` — must
        /// resolve to a single registered entry.
        entry: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Cmd::Report { run_dir }) = &args.cmd {
        return regenerate_report(run_dir);
    }
    if let Some(Cmd::Prepare { tool, entry }) = &args.cmd {
        return prepare_one(&args, tool, entry);
    }

    // Make TS_PROJECT_ROOT available to subprocesses (and to tool.toml's
    // ${VAR} expansion). Equals the directory the runner was invoked from —
    // i.e. the project root containing examples/, tools/, runs/.
    if std::env::var_os("TS_PROJECT_ROOT").is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            std::env::set_var("TS_PROJECT_ROOT", cwd);
        }
    }

    let examples_dir = args
        .examples
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", args.examples.display()))?;
    let tools_dir = args
        .tools
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", args.tools.display()))?;

    let examples = discover::find_examples(&examples_dir)?;
    let tools = discover::find_tools(&tools_dir)?;

    if examples.is_empty() {
        eprintln!("No examples found under {}", examples_dir.display());
        return Ok(());
    }
    if tools.is_empty() {
        eprintln!("No tools found under {}", tools_dir.display());
        return Ok(());
    }

    let run_id = format!(
        "run-{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        std::process::id()
    );
    let run_dir = args.runs.join(&run_id);
    std::fs::create_dir_all(&run_dir)
        .with_context(|| format!("creating {}", run_dir.display()))?;
    let run_dir = run_dir
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", run_dir.display()))?;

    // Build optional filters.
    let tool_set: Option<HashSet<&str>> = if args.tools_filter.is_empty() {
        None
    } else {
        Some(args.tools_filter.iter().map(String::as_str).collect())
    };
    let entry_globs: Vec<Pattern> = args
        .entries_filter
        .iter()
        .map(|s| Pattern::new(s).with_context(|| format!("invalid --entry glob `{}`", s)))
        .collect::<Result<_>>()?;

    // Build the (tool, example, entry) cartesian, filtered if either filter is set.
    let mut tasks: Vec<(&discover::Tool, &discover::Example, &str)> = Vec::new();
    for ex in &examples {
        for entry in &ex.entries {
            let entry_id = format!("{}/{}/{}", ex.feature, ex.dir, entry);
            if !entry_globs.is_empty()
                && !entry_globs.iter().any(|p| p.matches(&entry_id))
            {
                continue;
            }
            for tool in &tools {
                if let Some(set) = &tool_set {
                    if !set.contains(tool.name.as_str()) {
                        continue;
                    }
                }
                tasks.push((tool, ex, entry.as_str()));
            }
        }
    }

    if tasks.is_empty() {
        eprintln!("No tasks match the given filters (--tool / --entry).");
        return Ok(());
    }

    let parallel = args.parallel.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });
    eprintln!("Running {} task(s) with parallelism = {}", tasks.len(), parallel);

    // Snapshot run-level metadata BEFORE the work pool starts: timestamps,
    // host info, per-tool versions. version_command runs once per tool here
    // (best-effort; failure → version = None, doesn't block the run).
    let run_started = SystemTime::now();
    let run_started_unix_secs = run_started
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let host_info = host::collect();
    let tools_meta: Vec<report::ToolMeta> = tools
        .iter()
        .map(|t| report::ToolMeta {
            name: t.name.clone(),
            // P41: use raw ${TS_*} form (un-expanded) for results.json — avoids
            // embedding host-specific absolute paths in published artifacts.
            command: t.command_raw.clone(),
            timeout_secs: t.timeout_secs,
            entry_mode: format!("{:?}", t.entry_mode).to_lowercase(),
            extra_cargo_deps: t.extra_cargo_deps.clone(),
            version: host::capture_version(&t.version_command),
        })
        .collect();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(parallel)
        .build()
        .context("building rayon thread pool")?;

    // Run all tasks in parallel; collect per-task results for the report.
    // println! inside the closure is internally synchronized; lines won't tear.
    let results: Vec<report::TaskResult> = pool.install(|| {
        tasks
            .par_iter()
            .map(|(tool, example, entry)| {
                let entry_id = format!("{}/{}/{}", example.feature, example.dir, entry);
                match exec::execute(tool, example, entry, &run_dir, args.keep_work_dir) {
                    Ok(r) => {
                        // For FAILED tasks (not timeouts — those are SIGKILL by
                        // us and the stderr is whatever the child managed to
                        // flush before being killed; never an external-fault
                        // signal we should swallow), peek into the captured
                        // stderr/stdout and apply the external-fault catalogue
                        // (report::classify_external_fault). On a hit, re-emit
                        // the task as UNKNOWN with the diagnostic tag in
                        // `error`, preserving raw_stdout/raw_stderr for audit.
                        //
                        // Per principles.md §六 "Oracle 责任 / 不冤枉" + the
                        // false-positive audit (docs/fixes/false-positive-audit-2026-05-11.md
                        // §4.1-4.5): runnable harness mismatch, single-file
                        // pipelines missing cargo deps, old-cargo edition
                        // refusal, vendor lint strictness and env corruption
                        // are runner / tool-pipeline-upstream faults, never
                        // tool-frontend rejections of the entry's Rust.
                        let mut status_str = match r.status {
                            exec::Status::Success => "SUCCESS",
                            exec::Status::Failed => "FAILED",
                        };
                        let mut external_fault: Option<&'static str> = None;
                        if matches!(r.status, exec::Status::Failed) && !r.timed_out {
                            // Read just the stderr; stdout passed for parity
                            // even though no current rule consults it.
                            let stderr_path = run_dir.join(&r.raw_stderr_rel);
                            let stdout_path = run_dir.join(&r.raw_stdout_rel);
                            let stderr_text =
                                std::fs::read_to_string(&stderr_path).unwrap_or_default();
                            let stdout_text =
                                std::fs::read_to_string(&stdout_path).unwrap_or_default();
                            if let Some(tag) = report::classify_external_fault(
                                &stderr_text,
                                &stdout_text,
                                r.exit_code,
                            ) {
                                external_fault = Some(tag);
                                status_str = "UNKNOWN";
                            }
                        }
                        let tag = match (status_str, r.timed_out) {
                            ("SUCCESS", _) => "SUCCESS",
                            ("UNKNOWN", _) => "UNKNOWN",
                            (_, true) => "TIMEOUT",
                            _ => "FAILED ",
                        };
                        let exit_part = if r.timed_out {
                            ", timed out".to_string()
                        } else if let Some(ef) = external_fault {
                            format!(", external_fault={}", ef)
                        } else {
                            match r.exit_code {
                                Some(c) if !matches!(r.status, exec::Status::Success) => {
                                    format!(", exit={}", c)
                                }
                                _ => String::new(),
                            }
                        };
                        println!(
                            "[{tag}] {tool} {entry_id} ({ms}ms{exit_part})",
                            tool = tool.name,
                            ms = r.duration_ms,
                        );
                        report::TaskResult {
                            entry_id,
                            tool: tool.name.clone(),
                            status: status_str.to_string(),
                            exit_code: r.exit_code,
                            duration_ms: r.duration_ms,
                            timed_out: r.timed_out,
                            raw_stdout: Some(r.raw_stdout_rel),
                            raw_stderr: Some(r.raw_stderr_rel),
                            error: external_fault.map(|s| format!("external_fault: {}", s)),
                        }
                    }
                    Err(e) => {
                        // Runner-internal failure (cp / render / spawn / IO etc).
                        // Not the tool's responsibility — classify as UNKNOWN.
                        let err_str = format!("{:#}", e);
                        println!(
                            "[UNKNOWN] {tool} {entry_id} : {err}",
                            tool = tool.name,
                            err = err_str,
                        );
                        report::TaskResult {
                            entry_id,
                            tool: tool.name.clone(),
                            status: "UNKNOWN".to_string(),
                            exit_code: None,
                            duration_ms: 0,
                            timed_out: false,
                            raw_stdout: None,
                            raw_stderr: None,
                            error: Some(err_str),
                        }
                    }
                }
            })
            .collect()
    });

    let run_finished = SystemTime::now();
    let run_finished_unix_secs = run_finished
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let run_meta = report::RunMeta {
        run_id: run_id.clone(),
        run_started_at: host::iso8601_utc(run_started),
        run_finished_at: host::iso8601_utc(run_finished),
        run_started_unix_secs,
        run_finished_unix_secs,
        host: host_info,
        parallelism: parallel,
        tools: tools_meta,
    };

    report::write_results_json(&run_dir, &run_meta, &results)?;
    report::write_report_md(&run_dir, &run_meta, &results)?;

    let s = results.iter().filter(|r| r.status == "SUCCESS").count();
    let f = results.iter().filter(|r| r.status == "FAILED").count();
    let u = results.iter().filter(|r| r.status == "UNKNOWN").count();
    println!("---");
    println!(
        "Total: {} succeeded / {} failed / {} unknown / {} total",
        s, f, u, results.len()
    );
    println!("run_dir: {}", run_dir.display());
    println!("report:  {}", run_dir.join("report.md").display());
    Ok(())
}

/// `runner prepare <tool> <entry>`: prepare a single (tool, entry) work
/// directory without spawning the tool subprocess. Used to inspect what the
/// runner does "intrusively" before the tool runs — the rendered harness,
/// patched Cargo.toml, lib-mode lib.rs → __ts_inner.rs rename, etc. The
/// preserved work_dir path is printed for the user to `cd` into.
///
/// `entry` must be the full `<feature>/<dir>/<entry-fn>` id (the same form
/// reported in results.json `entry_id`), so this remains unambiguous in the
/// presence of duplicate entry-fn names across examples.
fn prepare_one(args: &Args, tool_name: &str, entry_id: &str) -> Result<()> {
    if std::env::var_os("TS_PROJECT_ROOT").is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            std::env::set_var("TS_PROJECT_ROOT", cwd);
        }
    }

    let examples_dir = args
        .examples
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", args.examples.display()))?;
    let tools_dir = args
        .tools
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", args.tools.display()))?;

    let examples = discover::find_examples(&examples_dir)?;
    let tools = discover::find_tools(&tools_dir)?;

    let tool = tools
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| anyhow::anyhow!("no tool named `{}` under {}", tool_name, tools_dir.display()))?;

    let mut matches: Vec<(&discover::Example, &str)> = Vec::new();
    for ex in &examples {
        for entry in &ex.entries {
            let id = format!("{}/{}/{}", ex.feature, ex.dir, entry);
            if id == entry_id {
                matches.push((ex, entry.as_str()));
            }
        }
    }
    if matches.is_empty() {
        return Err(anyhow::anyhow!(
            "no entry matches `{}`. Use the full `<feature>/<dir>/<entry-fn>` form.",
            entry_id
        ));
    }
    if matches.len() > 1 {
        return Err(anyhow::anyhow!(
            "entry id `{}` is ambiguous ({} matches) — should be impossible for an exact-id match",
            entry_id,
            matches.len()
        ));
    }
    let (example, entry) = matches[0];

    let run_id = format!(
        "prepare-{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        std::process::id()
    );
    let run_dir = args.runs.join(&run_id);
    std::fs::create_dir_all(&run_dir)
        .with_context(|| format!("creating {}", run_dir.display()))?;
    let run_dir = run_dir
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", run_dir.display()))?;

    let work_dir = exec::prepare(tool, example, entry, &run_dir)?;

    // Print a tour of the prepared state so the user knows what to inspect.
    println!("---");
    println!("prepared work_dir: {}", work_dir.display());
    println!();
    println!("tool         : {}", tool.name);
    println!("entry_id     : {}/{}/{}", example.feature, example.dir, entry);
    println!("entry_mode   : {:?}", tool.entry_mode);
    let target_rel = example.target_path.strip_prefix(&example.root).unwrap_or(&example.target_path);
    let target_rel_display = if target_rel.as_os_str().is_empty() {
        std::path::PathBuf::from(".")
    } else {
        target_rel.to_path_buf()
    };
    println!("target_path  : {}", target_rel_display.display());
    println!();
    println!("Files written by the runner (intrusive layer):");
    let target_in_workdir = work_dir.join(target_rel);
    match tool.entry_mode {
        discover::EntryMode::Bin => {
            println!("  + {}/src/bin/__ts_harness.rs   (rendered harness — new file)",
                target_in_workdir.display());
        }
        discover::EntryMode::Lib => {
            println!("  + {}/src/lib.rs                (rendered harness — replaces original lib)",
                target_in_workdir.display());
            println!("  + {}/src/__ts_inner.rs         (original lib.rs renamed here, re-exported by harness)",
                target_in_workdir.display());
        }
    }
    if !tool.extra_cargo_deps.is_empty() {
        println!("  ~ {}/Cargo.toml                  ([dependencies] patched with extra_cargo_deps:",
            target_in_workdir.display());
        for dep in &tool.extra_cargo_deps {
            println!("        {}", dep);
        }
        println!("    )");
    }
    println!();
    println!("At spawn time (skipped by `prepare`), the runner would also:");
    println!("  - strip every TS_* envvar from the child env");
    println!("  - inject TS_ENTRY_FN={}", entry);
    println!("  - inject TS_TARGET_CRATE={}", example.crate_name.replace('-', "_"));
    if !example.env.is_empty() {
        println!("  - inject hirusttest [env] vars:");
        for (k, v) in &example.env {
            println!("        {}={}", k, v);
        }
    }
    println!("  - cwd = {}", target_in_workdir.display());
    println!("  - argv (raw, ${{TS_*}} un-expanded) = {:?}", tool.command_raw);
    println!("  - argv (expanded) = {:?}", tool.command);
    println!();
    println!("To inspect: cd {}", target_in_workdir.display());
    println!("To clean up later: rm -rf {}", run_dir.display());
    Ok(())
}

/// `runner report <run-dir>`: regenerate report.md from an existing run's
/// results.json. Used to refresh report layout on already-collected raw data
/// without re-running tools.
fn regenerate_report(run_dir: &PathBuf) -> Result<()> {
    let run_dir = run_dir
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", run_dir.display()))?;
    let json_path = run_dir.join("results.json");
    let text = std::fs::read_to_string(&json_path)
        .with_context(|| format!("reading {}", json_path.display()))?;
    let parsed: report::ResultsFile =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", json_path.display()))?;
    report::write_report_md(&run_dir, &parsed.meta, &parsed.results)?;
    println!("regenerated: {}", run_dir.join("report.md").display());
    Ok(())
}
