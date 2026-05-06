use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use tera::{Context as TeraContext, Tera};

use crate::discover::{Example, Tool};

pub enum Status {
    Success,
    Failed,
}

pub struct ExecResult {
    pub status: Status,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
}

/// Run one (tool, example, entry) execution end-to-end:
/// 1. git worktree add into runs/<run_id>/work/<exec_id>/
/// 2. render harness.rs.tera into <worktree>/<target_path>/src/bin/__ts_harness.rs
/// 3. spawn tool.command with cwd = <worktree>/<target_path>
/// 4. capture stdout/stderr/exit; persist into runs/<run_id>/raw/<tool>/
/// 5. git worktree remove --force
pub fn execute(
    tool: &Tool,
    example: &Example,
    entry: &str,
    run_dir: &Path,
) -> Result<ExecResult> {
    let exec_id = format!(
        "{}__{}__{}__{}",
        sanitize(&tool.name),
        sanitize(&example.feature),
        sanitize(&example.dir),
        sanitize(entry),
    );
    let work_dir = run_dir.join("work").join(&exec_id);

    let testsuite_root = std::env::current_dir().context("getting cwd")?;
    let testsuite_root = testsuite_root.canonicalize().context("canonicalizing cwd")?;

    // 1. git worktree add
    let status = Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(&work_dir)
        .arg("HEAD")
        .current_dir(&testsuite_root)
        .status()
        .context("spawning `git worktree add`")?;
    if !status.success() {
        anyhow::bail!("`git worktree add` failed for {}", work_dir.display());
    }

    // The example's target_path is absolute under testsuite_root; remap to inside the worktree.
    let target_rel = example
        .target_path
        .strip_prefix(&testsuite_root)
        .with_context(|| {
            format!(
                "example target {} is not under {}",
                example.target_path.display(),
                testsuite_root.display()
            )
        })?;
    let target_in_worktree = work_dir.join(target_rel);

    // 2. Render harness
    let mut tera = Tera::default();
    tera.add_raw_template("harness", &tool.harness_template)
        .context("registering harness template")?;
    let mut ctx = TeraContext::new();
    ctx.insert("target_crate_name", &example.crate_name);
    ctx.insert("entry_fn", entry);
    let rendered = tera
        .render("harness", &ctx)
        .context("rendering harness template")?;

    let bin_dir = target_in_worktree.join("src").join("bin");
    std::fs::create_dir_all(&bin_dir)
        .with_context(|| format!("creating {}", bin_dir.display()))?;
    let harness_path = bin_dir.join("__ts_harness.rs");
    std::fs::write(&harness_path, rendered)
        .with_context(|| format!("writing {}", harness_path.display()))?;

    // 3. Run command
    let start = Instant::now();
    let mut cmd = Command::new(&tool.command[0]);
    cmd.args(&tool.command[1..]);
    cmd.current_dir(&target_in_worktree);

    let output = cmd
        .output()
        .with_context(|| format!("running {:?} in {}", tool.command, target_in_worktree.display()))?;
    let duration_ms = start.elapsed().as_millis();

    // 4. Persist raw outputs
    let raw_dir = run_dir.join("raw").join(&tool.name);
    std::fs::create_dir_all(&raw_dir)
        .with_context(|| format!("creating {}", raw_dir.display()))?;
    let slug = format!(
        "{}__{}__{}",
        sanitize(&example.feature),
        sanitize(&example.dir),
        sanitize(entry),
    );
    std::fs::write(raw_dir.join(format!("{}.stdout", slug)), &output.stdout)?;
    std::fs::write(raw_dir.join(format!("{}.stderr", slug)), &output.stderr)?;
    std::fs::write(
        raw_dir.join(format!("{}.exit", slug)),
        format!("{:?}\n", output.status.code()),
    )?;

    // 5. Cleanup worktree
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&work_dir)
        .current_dir(&testsuite_root)
        .status();

    let exit_code = output.status.code();
    let status = if output.status.success() {
        Status::Success
    } else {
        Status::Failed
    };
    Ok(ExecResult {
        status,
        exit_code,
        duration_ms,
    })
}

/// Replace path-unsafe characters in IDs with underscores for filesystem use.
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | ' ' => '_',
            c => c,
        })
        .collect()
}
