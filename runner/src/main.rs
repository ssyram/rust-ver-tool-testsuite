use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Canonicalize so absolute paths flow through.
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

    let mut total = 0usize;
    let mut succeeded = 0usize;
    let mut failed = 0usize;

    for example in &examples {
        for entry in &example.entries {
            for tool in &tools {
                let result = exec::execute(tool, example, entry, &run_dir)?;
                total += 1;
                let tag = match result.status {
                    exec::Status::Success => {
                        succeeded += 1;
                        "SUCCESS"
                    }
                    exec::Status::Failed => {
                        failed += 1;
                        "FAILED "
                    }
                };
                let exit_part = match result.exit_code {
                    Some(c) if !matches!(result.status, exec::Status::Success) => {
                        format!(", exit={}", c)
                    }
                    _ => String::new(),
                };
                println!(
                    "[{tag}] {tool} {feat}/{dir}/{entry} ({ms}ms{exit_part})",
                    tool = tool.name,
                    feat = example.feature,
                    dir = example.dir,
                    entry = entry,
                    ms = result.duration_ms,
                );
            }
        }
    }

    println!("---");
    println!(
        "Total: {} succeeded / {} failed / {} total",
        succeeded, failed, total
    );
    println!("run_dir: {}", run_dir.display());
    Ok(())
}
