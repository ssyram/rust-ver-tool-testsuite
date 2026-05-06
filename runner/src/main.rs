use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

mod discover;
mod exec;

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

    let succeeded = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);

    pool.install(|| {
        tasks.par_iter().for_each(|(tool, example, entry)| {
            match exec::execute(tool, example, entry, &run_dir) {
                Ok(r) => {
                    let tag = match r.status {
                        exec::Status::Success => {
                            succeeded.fetch_add(1, Ordering::Relaxed);
                            "SUCCESS"
                        }
                        exec::Status::Failed => {
                            failed.fetch_add(1, Ordering::Relaxed);
                            "FAILED "
                        }
                    };
                    let exit_part = match r.exit_code {
                        Some(c) if !matches!(r.status, exec::Status::Success) => {
                            format!(", exit={}", c)
                        }
                        _ => String::new(),
                    };
                    // println! is internally synchronized; lines won't tear.
                    println!(
                        "[{tag}] {tool} {feat}/{dir}/{entry} ({ms}ms{exit_part})",
                        tool = tool.name,
                        feat = example.feature,
                        dir = example.dir,
                        entry = entry,
                        ms = r.duration_ms,
                    );
                }
                Err(e) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    println!(
                        "[ERROR ] {tool} {feat}/{dir}/{entry} : {err:#}",
                        tool = tool.name,
                        feat = example.feature,
                        dir = example.dir,
                        entry = entry,
                        err = e,
                    );
                }
            }
        });
    });

    let s = succeeded.load(Ordering::Relaxed);
    let f = failed.load(Ordering::Relaxed);
    let total = s + f;
    println!("---");
    println!("Total: {} succeeded / {} failed / {} total", s, f, total);
    println!("run_dir: {}", run_dir.display());
    Ok(())
}
