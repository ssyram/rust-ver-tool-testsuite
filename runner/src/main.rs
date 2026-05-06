use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod discover;
mod exec;
mod report;

#[derive(Parser, Debug)]
#[command(name = "runner")]
#[command(about = "Rust verification tool feature-coverage runner")]
struct Args {
    /// Path to examples directory
    #[arg(long, default_value = "examples")]
    examples: PathBuf,

    /// Path to tools directory
    #[arg(long, default_value = "tools")]
    tools: PathBuf,

    /// Path to runs output directory
    #[arg(long, default_value = "runs")]
    runs: PathBuf,

    /// Max parallel executions (default: number of CPU cores)
    #[arg(long)]
    parallel: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();

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
        "run-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    );
    let run_dir = args.runs.join(&run_id);
    std::fs::create_dir_all(&run_dir)
        .with_context(|| format!("creating {}", run_dir.display()))?;
    let run_dir = run_dir
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", run_dir.display()))?;

    // Build the (tool, example, entry) cartesian.
    let mut tasks: Vec<(&discover::Tool, &discover::Example, &str)> = Vec::new();
    for ex in &examples {
        for entry in &ex.entries {
            for tool in &tools {
                tasks.push((tool, ex, entry.as_str()));
            }
        }
    }

    let parallel = args.parallel.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });
    eprintln!("Running {} task(s) with parallelism = {}", tasks.len(), parallel);

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
                match exec::execute(tool, example, entry, &run_dir) {
                    Ok(r) => {
                        let (tag, status_str) = match r.status {
                            exec::Status::Success => ("SUCCESS", "SUCCESS"),
                            exec::Status::Failed => ("FAILED ", "FAILED"),
                        };
                        let exit_part = match r.exit_code {
                            Some(c) if !matches!(r.status, exec::Status::Success) => {
                                format!(", exit={}", c)
                            }
                            _ => String::new(),
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
                            raw_stdout: Some(r.raw_stdout_rel),
                            raw_stderr: Some(r.raw_stderr_rel),
                        }
                    }
                    Err(e) => {
                        println!(
                            "[ERROR ] {tool} {entry_id} : {err:#}",
                            tool = tool.name,
                            err = e,
                        );
                        report::TaskResult {
                            entry_id,
                            tool: tool.name.clone(),
                            status: "FAILED".to_string(),
                            exit_code: None,
                            duration_ms: 0,
                            raw_stdout: None,
                            raw_stderr: None,
                        }
                    }
                }
            })
            .collect()
    });

    report::write_results_json(&run_dir, &run_id, &results)?;
    report::write_report_md(&run_dir, &run_id, &results)?;

    let s = results.iter().filter(|r| r.status == "SUCCESS").count();
    let f = results.len() - s;
    println!("---");
    println!("Total: {} succeeded / {} failed / {} total", s, f, results.len());
    println!("run_dir: {}", run_dir.display());
    println!("report:  {}", run_dir.join("report.md").display());
    Ok(())
}
