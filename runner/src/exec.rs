use anyhow::{anyhow, Context, Result};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tera::{Context as TeraContext, Tera};
use wait_timeout::ChildExt;

use crate::discover::{EntryMode, Example, Tool};

pub enum Status {
    Success,
    Failed,
}

pub struct ExecResult {
    pub status: Status,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    /// True if the subprocess was killed because it exceeded `tool.timeout_secs`.
    /// `status` is always `Failed` in this case.
    pub timed_out: bool,
    /// Relative path (from run_dir) to the captured stdout file.
    pub raw_stdout_rel: String,
    /// Relative path (from run_dir) to the captured stderr file.
    pub raw_stderr_rel: String,
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
    keep_work_dir: bool,
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
    // Exclude `target/` to avoid copying build artifacts. Exclude `Cargo.lock`
    // because (a) keeping a stale lock under a tool with an older toolchain
    // (e.g. prusti's pinned 2023-08-15 nightly cannot parse v4 lockfiles
    // written by recent stable cargo) breaks builds, and (b) for a feature-
    // coverage screening run, fresh dep resolution per task is what we want
    // anyway — it gives every tool the same view of the dependency graph in
    // the current run.
    copy_dir_excluding(&example.root, &work_dir, &["target", "Cargo.lock"])
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

    // 1b. If the tool declares extra cargo dependencies, append them to the
    // working-copy Cargo.toml's [dependencies] section. This supports tools
    // (e.g. Creusot) that require the example crate to depend on a tool-
    // specific support crate. Only the working copy is touched — the original
    // example is not modified.
    if !tool.extra_cargo_deps.is_empty() {
        let cargo_toml = target_in_workdir.join("Cargo.toml");
        patch_cargo_deps(&cargo_toml, &tool.extra_cargo_deps)
            .with_context(|| format!("patching {}", cargo_toml.display()))?;
    }

    // 2. Render harness
    let mut tera = Tera::default();
    tera.add_raw_template("harness", &tool.harness_template)
        .context("registering harness template")?;
    let mut ctx = TeraContext::new();
    // cargo's standard convention: a package name like "bool-bitwise-op"
    // becomes the Rust crate identifier "bool_bitwise_op". `target_crate_name`
    // is consumed in Rust path position by every harness template, so we
    // must hand over the identifier form, not the cargo manifest name.
    let crate_ident = example.crate_name.replace('-', "_");
    ctx.insert("target_crate_name", &crate_ident);
    ctx.insert("entry_fn", entry);
    // `entry_args` is the call-site argument list (a Rust expression list,
    // empty for entries without a `[runnable.<entry>]` table — back-compat
    // with the original zero-arg harness contract). See
    // discover.rs::Example::entry_args + detailed-design.md §一 ("Schema 向后
    // 兼容性") and false-positive-audit-2026-05-11.md §4.1 — without this,
    // bin-mode harnesses on the 15 runnable entries fail with E0061 across
    // ~9 tools (134 audited false positives).
    let entry_args: &str = example
        .entry_args
        .get(entry)
        .map(String::as_str)
        .unwrap_or("");
    ctx.insert("entry_args", entry_args);
    let rendered = tera
        .render("harness", &ctx)
        .context("rendering harness template")?;

    let src_dir = target_in_workdir.join("src");
    match tool.entry_mode {
        EntryMode::Bin => {
            // Render to src/bin/__ts_harness.rs alongside the original lib.
            let bin_dir = src_dir.join("bin");
            std::fs::create_dir_all(&bin_dir)
                .with_context(|| format!("creating {}", bin_dir.display()))?;
            let harness_path = bin_dir.join("__ts_harness.rs");
            std::fs::write(&harness_path, &rendered)
                .with_context(|| format!("writing {}", harness_path.display()))?;
        }
        EntryMode::Lib => {
            // Replace the lib entry: rename the original src/lib.rs to
            // src/__ts_inner.rs, then write the harness as the new src/lib.rs.
            // The harness template is expected to contain `mod __ts_inner;
            // pub use __ts_inner::*;` so that the original lib's public items
            // remain reachable as `<crate>::*`.
            let original_lib = src_dir.join("lib.rs");
            let inner_lib = src_dir.join("__ts_inner.rs");
            if !original_lib.exists() {
                return Err(anyhow!(
                    "entry_mode = \"lib\" requires {} to exist",
                    original_lib.display()
                ));
            }
            std::fs::rename(&original_lib, &inner_lib).with_context(|| {
                format!(
                    "renaming {} → {}",
                    original_lib.display(),
                    inner_lib.display()
                )
            })?;
            std::fs::write(&original_lib, &rendered)
                .with_context(|| format!("writing {}", original_lib.display()))?;
        }
    }

    // 3. Run command — per-(tool, entry) timeout via wait_timeout. The child
    // is started in its own process group (setpgid via process_group(0)) so
    // that on timeout we can SIGKILL the entire group, including grandchildren
    // such as cbmc spawned by cargo-kani. Without group-kill, the grandchild
    // keeps the stdout/stderr pipe fds open and our reader threads block on
    // read_to_end forever, deadlocking the runner thread.
    //
    // stdout/stderr are read by background threads to avoid blocking the
    // parent on pipe buffer fill while we wait.
    let start = Instant::now();
    let mut command_builder = Command::new(&tool.command[0]);
    command_builder
        .args(&tool.command[1..])
        .current_dir(&target_in_workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Strip all TS_* environment variables before spawning the child. These
    // are runner-internal config (sourced from .env, used for ${VAR}
    // expansion in tool.toml) and must not leak into tool processes — some
    // tools (e.g. prusti) treat any envvar matching their own prefix as
    // a config flag and panic on unknown names. The TS_PROJECT_ROOT case
    // is also handled by stripping; tool.toml has already had it expanded
    // by the runner before this point.
    let ts_vars: Vec<String> = std::env::vars()
        .map(|(k, _)| k)
        .filter(|k| k.starts_with("TS_"))
        .collect();
    for k in &ts_vars {
        command_builder.env_remove(k);
    }
    // Inject runner-managed metadata so oracle scripts can verify the entry
    // function appears in the tool's product (closes the type-2 silent path
    // where a translation tool may fully skip an item without emitting any
    // sentinel marker — the item simply doesn't appear in the .lean/.v/.fst
    // output. Without this guard, exit-0 tools could silently drop functions).
    // These are set AFTER env_remove so they survive into the child env.
    command_builder.env("TS_ENTRY_FN", entry);
    command_builder.env("TS_TARGET_CRATE", &crate_ident);

    // Inject per-example env vars declared in hirusttest.toml `[env]` table.
    // Used when the example needs specific runtime environment (e.g.
    // `RUSTFLAGS=--cap-lints=warn` for entries that depend on a vendored
    // crate whose `#![deny(...)]` would otherwise fire on the tool's
    // (newer) rustc — a vendor-crate code-style choice that does not
    // reflect any tool capability boundary. See principles.md §四 A: the
    // hirusttest signal file does not change the example's cargo bytes;
    // the runner reads this declaration and applies it externally at spawn.
    // Applied AFTER env_remove + TS_* injection so example declarations
    // take precedence over any prior runner-internal env; but a tool can
    // still override at its own tool.toml command level if it needs to
    // (no example-level env will reach into tool.toml ${VAR} expansion).
    for (k, v) in &example.env {
        command_builder.env(k, v);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command_builder.process_group(0);
    }
    let mut child = command_builder
        .spawn()
        .with_context(|| format!("spawning {:?} in {}", tool.command, target_in_workdir.display()))?;

    let child_pid = child.id() as i32;
    let mut child_stdout = child.stdout.take().expect("piped stdout");
    let mut child_stderr = child.stderr.take().expect("piped stderr");
    let stdout_reader = std::thread::spawn(move || -> Vec<u8> {
        let mut buf = Vec::new();
        let _ = child_stdout.read_to_end(&mut buf);
        buf
    });
    let stderr_reader = std::thread::spawn(move || -> Vec<u8> {
        let mut buf = Vec::new();
        let _ = child_stderr.read_to_end(&mut buf);
        buf
    });

    let timeout = Duration::from_secs(tool.timeout_secs);
    let (exit_status, timed_out) = match child
        .wait_timeout(timeout)
        .with_context(|| format!("waiting on {:?}", tool.command))?
    {
        Some(s) => (s, false),
        None => {
            // Kill the whole process group so cbmc / sub-cargo / etc. die too.
            #[cfg(unix)]
            unsafe {
                libc::kill(-child_pid as libc::pid_t, libc::SIGKILL);
            }
            #[cfg(not(unix))]
            let _ = child.kill();
            let s = child
                .wait()
                .with_context(|| format!("reaping killed {:?}", tool.command))?;
            (s, true)
        }
    };
    let stdout_buf = stdout_reader.join().unwrap_or_default();
    let stderr_buf = stderr_reader.join().unwrap_or_default();
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
    let raw_stdout_rel = format!("raw/{}/{}.stdout", tool.name, slug);
    let raw_stderr_rel = format!("raw/{}/{}.stderr", tool.name, slug);
    let raw_exit_rel = format!("raw/{}/{}.exit", tool.name, slug);
    std::fs::write(run_dir.join(&raw_stdout_rel), &stdout_buf)?;
    std::fs::write(run_dir.join(&raw_stderr_rel), &stderr_buf)?;
    let exit_marker = if timed_out {
        format!("__ts_timeout (after {} s)\n", tool.timeout_secs)
    } else {
        format!("{:?}\n", exit_status.code())
    };
    std::fs::write(run_dir.join(&raw_exit_rel), exit_marker)?;

    // 5. Cleanup work dir. Best-effort: cleanup failure leaves the work_dir
    // behind but does not invalidate captured raw outputs (see §7.6 spec note).
    // P45: skip cleanup when --keep-work-dir is set so the user can inspect
    // the framework's intermediate state (rendered harness / patched
    // Cargo.toml / __ts_inner.rs rename / etc.).
    if !keep_work_dir {
        if let Err(e) = std::fs::remove_dir_all(&work_dir) {
            eprintln!("[warn] cleanup {} failed: {:#}", work_dir.display(), e);
        }
    } else {
        eprintln!("[keep-work-dir] preserved: {}", work_dir.display());
    }

    let exit_code = exit_status.code();
    let status = if !timed_out && exit_status.success() {
        Status::Success
    } else {
        Status::Failed
    };
    Ok(ExecResult {
        status,
        exit_code,
        duration_ms,
        timed_out,
        raw_stdout_rel,
        raw_stderr_rel,
    })
}

/// Prepare a (tool, entry) work directory **without spawning the tool**.
/// Mirror of `execute()` steps 1-2 only:
///   1. cp example → work_dir (skip target/ + Cargo.lock)
///   2. patch Cargo.toml with extra_cargo_deps (if any)
///   3. render harness.rs.tera + place at src/bin/__ts_harness.rs (Bin mode)
///      OR rename src/lib.rs → src/__ts_inner.rs and render harness as new
///      src/lib.rs (Lib mode)
///
/// Returns the prepared work_dir path. Caller should NOT delete it — the
/// whole point of `prepare` is to let the user inspect it.
///
/// Side effects to surface to the user (the `runner prepare` printout):
///   - prepared work_dir path
///   - rendered files list
///   - effective tool.command (after `${TS_*}` expansion + envvar injections
///     that will happen at spawn time — TS_ENTRY_FN / TS_TARGET_CRATE)
pub fn prepare(
    tool: &Tool,
    example: &Example,
    entry: &str,
    run_dir: &Path,
) -> Result<std::path::PathBuf> {
    let exec_id = format!(
        "{}__{}__{}__{}",
        sanitize(&tool.name),
        sanitize(&example.feature),
        sanitize(&example.dir),
        sanitize(entry),
    );
    let work_dir = run_dir.join("work").join(&exec_id);

    if work_dir.exists() {
        std::fs::remove_dir_all(&work_dir)
            .with_context(|| format!("clearing {}", work_dir.display()))?;
    }
    copy_dir_excluding(&example.root, &work_dir, &["target", "Cargo.lock"])
        .with_context(|| format!("copying {} → {}", example.root.display(), work_dir.display()))?;

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

    if !tool.extra_cargo_deps.is_empty() {
        let cargo_toml = target_in_workdir.join("Cargo.toml");
        patch_cargo_deps(&cargo_toml, &tool.extra_cargo_deps)
            .with_context(|| format!("patching {}", cargo_toml.display()))?;
    }

    let mut tera = Tera::default();
    tera.add_raw_template("harness", &tool.harness_template)
        .context("registering harness template")?;
    let mut ctx = TeraContext::new();
    let crate_ident = example.crate_name.replace('-', "_");
    ctx.insert("target_crate_name", &crate_ident);
    ctx.insert("entry_fn", entry);
    let entry_args: &str = example
        .entry_args
        .get(entry)
        .map(String::as_str)
        .unwrap_or("");
    ctx.insert("entry_args", entry_args);
    let rendered = tera
        .render("harness", &ctx)
        .context("rendering harness template")?;

    let src_dir = target_in_workdir.join("src");
    match tool.entry_mode {
        EntryMode::Bin => {
            let bin_dir = src_dir.join("bin");
            std::fs::create_dir_all(&bin_dir)
                .with_context(|| format!("creating {}", bin_dir.display()))?;
            let harness_path = bin_dir.join("__ts_harness.rs");
            std::fs::write(&harness_path, &rendered)
                .with_context(|| format!("writing {}", harness_path.display()))?;
        }
        EntryMode::Lib => {
            let original_lib = src_dir.join("lib.rs");
            let inner_lib = src_dir.join("__ts_inner.rs");
            if !original_lib.exists() {
                return Err(anyhow!(
                    "entry_mode = \"lib\" requires {} to exist",
                    original_lib.display()
                ));
            }
            std::fs::rename(&original_lib, &inner_lib).with_context(|| {
                format!(
                    "renaming {} → {}",
                    original_lib.display(),
                    inner_lib.display()
                )
            })?;
            std::fs::write(&original_lib, &rendered)
                .with_context(|| format!("writing {}", original_lib.display()))?;
        }
    }

    Ok(work_dir)
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

/// Insert each `dep_line` (a `name = value` TOML fragment) into the
/// `[dependencies]` table of `cargo_path`. Uses a TOML AST so it handles
/// comments, sub-tables, sibling sections, and pre-existing keys correctly.
/// Existing keys with the same name are overwritten by the injected value.
fn patch_cargo_deps(cargo_path: &Path, dep_lines: &[String]) -> Result<()> {
    if dep_lines.is_empty() {
        return Ok(());
    }
    let content = std::fs::read_to_string(cargo_path)
        .with_context(|| format!("reading {}", cargo_path.display()))?;
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("parsing {} as TOML", cargo_path.display()))?;

    let deps_item = doc
        .entry("dependencies")
        .or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()));
    let deps = deps_item.as_table_mut().ok_or_else(|| {
        anyhow!(
            "[dependencies] in {} is not a table — refusing to patch",
            cargo_path.display()
        )
    })?;

    for line in dep_lines {
        let mini: toml_edit::DocumentMut = line
            .parse()
            .with_context(|| format!("parsing extra_cargo_deps entry `{}` as TOML", line))?;
        for (k, v) in mini.as_table().iter() {
            deps.insert(k, v.clone());
        }
    }

    std::fs::write(cargo_path, doc.to_string())
        .with_context(|| format!("writing {}", cargo_path.display()))?;
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
