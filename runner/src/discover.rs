use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A discovered example test crate.
pub struct Example {
    /// First-level directory under examples/. Free-form string (e.g., "vec", "hello").
    pub feature: String,
    /// Second-level directory name (the cargo project root dir name).
    pub dir: String,
    /// Absolute path to the example root (i.e., examples/<feature>/<dir>/).
    pub root: PathBuf,
    /// Absolute path to the target crate (== root + target_path).
    pub target_path: PathBuf,
    /// Cargo [package].name of the target crate.
    pub crate_name: String,
    /// Entry function names registered in testsuite.toml.
    pub entries: Vec<String>,
}

#[derive(Deserialize)]
struct TestsuiteToml {
    entries: Vec<String>,
    target_path: Option<String>,
}

#[derive(Deserialize)]
struct CargoToml {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
}

/// Walk examples/<feature>/<dir>/ two levels deep; collect every dir that
/// contains a testsuite.toml.
pub fn find_examples(examples_dir: &Path) -> Result<Vec<Example>> {
    let mut result = Vec::new();
    for entry in WalkDir::new(examples_dir).max_depth(2).min_depth(2) {
        let entry = entry.with_context(|| format!("walking {}", examples_dir.display()))?;
        if !entry.file_type().is_dir() {
            continue;
        }
        let dir = entry.path();
        let testsuite_toml = dir.join("testsuite.toml");
        if !testsuite_toml.exists() {
            continue;
        }

        let ts_text = fs::read_to_string(&testsuite_toml)
            .with_context(|| format!("reading {}", testsuite_toml.display()))?;
        let ts: TestsuiteToml = toml::from_str(&ts_text)
            .with_context(|| format!("parsing {}", testsuite_toml.display()))?;

        let target_rel = ts.target_path.as_deref().unwrap_or(".");
        let target_dir = dir.join(target_rel).canonicalize().with_context(|| {
            format!(
                "canonicalizing target_path '{}' under {}",
                target_rel,
                dir.display()
            )
        })?;

        let cargo_path = target_dir.join("Cargo.toml");
        let cargo_text = fs::read_to_string(&cargo_path)
            .with_context(|| format!("reading {}", cargo_path.display()))?;
        let cargo: CargoToml = toml::from_str(&cargo_text)
            .with_context(|| format!("parsing {}", cargo_path.display()))?;

        let rel = dir.strip_prefix(examples_dir).unwrap();
        let mut comps = rel.components();
        let feature = comps
            .next()
            .ok_or_else(|| anyhow!("example {} has no feature dir", dir.display()))?
            .as_os_str()
            .to_string_lossy()
            .to_string();
        let dir_name = comps
            .next()
            .ok_or_else(|| anyhow!("example {} has no inner dir", dir.display()))?
            .as_os_str()
            .to_string_lossy()
            .to_string();

        result.push(Example {
            feature,
            dir: dir_name,
            root: dir.to_path_buf(),
            target_path: target_dir,
            crate_name: cargo.package.name,
            entries: ts.entries,
        });
    }
    result.sort_by(|a, b| {
        (a.feature.as_str(), a.dir.as_str()).cmp(&(b.feature.as_str(), b.dir.as_str()))
    });
    Ok(result)
}

/// A discovered tool integration: tools/<name>/{tool.toml + harness.rs.tera}.
pub struct Tool {
    pub name: String,
    pub command: Vec<String>,
    /// Per-tool timeout in seconds. Parsed but not yet enforced — see §7.5; exec kills via SIGTERM/SIGKILL after this duration. TODO.
    #[allow(dead_code)]
    pub timeout_secs: u64,
    pub harness_template: String,
}

#[derive(Deserialize)]
struct ToolToml {
    command: Vec<String>,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
}

fn default_timeout() -> u64 {
    300
}

pub fn find_tools(tools_dir: &Path) -> Result<Vec<Tool>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(tools_dir)
        .with_context(|| format!("reading {}", tools_dir.display()))?
    {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_dir() {
            continue;
        }
        let dir = entry.path();
        let tool_toml_path = dir.join("tool.toml");
        let harness_path = dir.join("harness.rs.tera");
        if !tool_toml_path.exists() || !harness_path.exists() {
            continue;
        }

        let tool_text = fs::read_to_string(&tool_toml_path)
            .with_context(|| format!("reading {}", tool_toml_path.display()))?;
        let parsed: ToolToml = toml::from_str(&tool_text)
            .with_context(|| format!("parsing {}", tool_toml_path.display()))?;
        let template = fs::read_to_string(&harness_path)
            .with_context(|| format!("reading {}", harness_path.display()))?;

        let name = dir
            .file_name()
            .ok_or_else(|| anyhow!("bad tool dir {}", dir.display()))?
            .to_string_lossy()
            .to_string();

        result.push(Tool {
            name,
            command: parsed.command,
            timeout_secs: parsed.timeout_secs,
            harness_template: template,
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}
