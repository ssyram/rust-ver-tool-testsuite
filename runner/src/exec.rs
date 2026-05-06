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
/// 1. copy example crate dir into runs/<run_id>/work/<exec_id>/ (skip target/)
/// 2. render harness.rs.tera into <work>/<target_rel>/src/bin/__ts_harness.rs
/// 3. spawn tool.command with cwd = <work>/<target_rel>
/// 4. capture stdout/stderr/exit; persist into runs/<run_id>/raw/<tool>/
/// 5. remove work dir
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

    // 1. Copy example crate into work_dir (skip target/ to avoid copying build artifacts)
    if work_dir.exists() {
        std::fs::remove_dir_all(&work_dir)
            .with_context(|| format!("clearing {}", work_dir.display()))?;
    }
    copy_dir_excluding(&example.root, &work_dir, &["target"])
        .with_context(|| format!("copying {} → {}", example.root.display(), work_dir.display()))?;

    // The example's target_path is absolute under example.root; remap to inside work_dir.
    let target_rel = example
        .target_path
        .strip_prefix(&example.root)
        .with_context(|| {
            format!(
                "example target {} is not under example root {}",
                example.target_path.display(),
                example.root.display()
            )
        })?;
    let target_in_workdir = work_dir.join(target_rel);

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

    let bin_dir = target_in_workdir.join("src").join("bin");
    std::fs::create_dir_all(&bin_dir)
        .with_context(|| format!("creating {}", bin_dir.display()))?;
    let harness_path = bin_dir.join("__ts_harness.rs");
    std::fs::write(&harness_path, rendered)
        .with_context(|| format!("writing {}", harness_path.display()))?;

    // 3. Run command
    let start = Instant::now();
    let mut cmd = Command::new(&tool.command[0]);
    cmd.args(&tool.command[1..]);
    cmd.current_dir(&target_in_workdir);

    let output = cmd
        .output()
        .with_context(|| format!("running {:?} in {}", tool.command, target_in_workdir.display()))?;
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

    // 5. Cleanup work dir
    let _ = std::fs::remove_dir_all(&work_dir);

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

/// Recursively copy `src` to `dst`, skipping any directory whose file name is in `excludes`.
fn copy_dir_excluding(src: &Path, dst: &Path, excludes: &[&str]) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if excludes.iter().any(|x| *x == name_str.as_ref()) {
            continue;
        }
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(&name);
        if ty.is_dir() {
            copy_dir_excluding(&from, &to, excludes)?;
        } else if ty.is_symlink() {
            // Preserve symlink as-is (uncommon in cargo crates; safe fallback).
            let target = std::fs::read_link(&from)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(target, &to)?;
            #[cfg(not(unix))]
            std::fs::copy(&from, &to).map(|_| ())?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
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
