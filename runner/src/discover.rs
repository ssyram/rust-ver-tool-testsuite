use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Which of the two example schema tracks an Example was discovered as.
/// See `principles.md` §四 原则 A 双轨 schema and `detailed-design.md` §一.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaKind {
    /// Single-file track (default): `<example_dir>/hirusttest.toml`. Used for
    /// simple single-feature tests.
    SingleFile,
    /// Directory track: `<example_dir>/.hirusttest/config.toml` (+ optional
    /// auxiliary files in the same `.hirusttest/` dir). Reserved for external
    /// composite projects (e.g. vendor/x509-parser).
    Directory,
}

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
    /// Entry function names registered in the example's config.
    pub entries: Vec<String>,
    /// Per-entry default call arguments rendered as a Rust expression list
    /// (e.g., `"3i32, 4i32"` for an `add_two(a: i32, b: i32)` entry whose
    /// hirusttest.toml declares `[runnable.add_two] inputs = [[3, 4], …]`).
    /// Filled from `inputs[0]` of the `[runnable.<entry>]` table (single-file
    /// track only; see detailed-design.md §一). Entry not in the map → its
    /// harness invocation must remain zero-arg (back-compat: 146 non-runnable
    /// entries continue unchanged).
    ///
    /// Each Tera context insert at exec time uses the looked-up value; empty
    /// string when the entry has no runnable table. Without this, every
    /// runnable-corpus invocation mechanically failed on bin-mode tools as
    /// E0061 (cf. false-positive-audit-2026-05-11.md §2.1 / §4.1 — 134 FPs).
    pub entry_args: std::collections::HashMap<String, String>,
    /// Which schema track this example was discovered as. Read by future
    /// exec-stage stub injection (P-future); currently observable but unused
    /// — silence dead-code until the consumer lands.
    #[allow(dead_code)]
    pub schema_kind: SchemaKind,
    /// Absolute path to the `.hirusttest/` directory for directory-track
    /// examples (used by exec for stub injection etc.). `None` for
    /// single-file-track examples. See `schema_kind` note re. dead-code.
    #[allow(dead_code)]
    pub hirusttest_dir: Option<PathBuf>,
    /// Per-example env vars injected by runner at subprocess spawn time.
    /// Sourced from `[env]` table in hirusttest.toml (single-file track).
    ///
    /// Purpose: lets an example **declare** what runtime environment it
    /// requires for a fair build/test (e.g., `RUSTFLAGS=--cap-lints=warn`
    /// for entries that depend on a vendored crate whose `#![deny(...)]`
    /// fires on newer rustc — a vendor-crate code-style choice that would
    /// otherwise block cargo build on all cargo-pipeline tools without
    /// reflecting any tool capability boundary).
    ///
    /// Aligns with principles.md §四 A: the signal file's *presence* still
    /// does not change the example's cargo bytes — the example folder itself
    /// (src/ + Cargo.toml) is untouched. The runner reads this declaration
    /// and applies it externally at spawn. Aligns with §四 C: heterogeneous
    /// per-example needs sink into declarative data, not runner code paths.
    pub env: std::collections::HashMap<String, String>,
}

/// Common config schema shared by both tracks. v1 only reads `entries` +
/// `target_path` + `runnable.<entry_fn>`; the directory track reserves
/// additional fields (`entry_overrides`, `tools.<name>`, ...) for future
/// extensions. We do not set `deny_unknown_fields`, so unknown keys are
/// silently accepted — extension fields can land in `.hirusttest/config.toml`
/// without breaking deserialize.
#[derive(Deserialize)]
struct HirusttestToml {
    entries: Vec<String>,
    target_path: Option<String>,
    /// Optional `[runnable.<entry_fn>] inputs = [...] expected = [...]` table.
    /// See detailed-design.md §一 for the full schema. v1 reads `inputs[0]`
    /// as the default argument tuple for the harness call site, so
    /// bin-mode tools can compile their harness against runnable entries
    /// (whose fns take arguments) instead of failing with E0061.
    #[serde(default)]
    runnable: std::collections::HashMap<String, RunnableSpec>,
    /// Optional `[env]` table — per-example env vars injected by runner
    /// at subprocess spawn time. See `Example::env` doc for semantics &
    /// constitutional alignment (§四 A signal-file non-invasion + §四 C
    /// heterogeneity sinks to declarative data).
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
}

/// `[runnable.<entry_fn>]` section. Only `inputs` is consumed by the runner
/// for harness rendering today; `expected` is reserved for future consistency
/// tooling (cf. hax-lean-consistency-design-2026-05-11.md §2).
#[derive(Deserialize)]
#[allow(dead_code)]
struct RunnableSpec {
    /// Each inner array is one set of actual arguments matching the fn
    /// signature order. v1 uses `inputs[0]` only.
    inputs: Vec<toml::Value>,
    /// Expected return value per inputs row. Read by future consistency
    /// tools, not by this runner (kept here so the field doesn't get
    /// rejected if `deny_unknown_fields` is ever enabled).
    #[serde(default)]
    expected: Option<toml::Value>,
}

#[derive(Deserialize)]
struct CargoToml {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
}

/// Marker file: if present in a directory, that directory and its entire
/// subtree are skipped — neither the directory itself nor any of its descendants
/// are considered as entries. Useful for vendor/ submodules that contain
/// Cargo workspaces unrelated to the testsuite, or for scratch dirs.
const SKIP_MARKER: &str = ".no-hirusttest";

/// Name of the directory-track config root. When this directory exists at an
/// example dir's top level, the example is treated as directory-track.
const DIR_TRACK_DIRNAME: &str = ".hirusttest";

/// Single-file-track config filename.
const SINGLE_FILE_TRACK_NAME: &str = "hirusttest.toml";

/// Directory-track config filename (lives inside `.hirusttest/`).
const DIR_TRACK_CONFIG_NAME: &str = "config.toml";

/// Walk examples/ at unlimited depth; collect every directory that holds either
/// a single-file-track signal (`hirusttest.toml`) or a directory-track signal
/// (`.hirusttest/config.toml`). A directory containing `.no-hirusttest` is
/// skipped along with its entire subtree. A `.hirusttest/` directory is never
/// recursed into — its contents are auxiliary files for its parent example,
/// not nested examples.
///
/// Disambiguation rules (per principles §四 原则 A 双轨 schema and
/// detailed-design §一):
///   1. `.hirusttest/config.toml` exists → directory track.
///   2. else `hirusttest.toml` exists → single-file track.
///   3. else not an example.
///   4. both `hirusttest.toml` and `.hirusttest/` present → return Err
///      (ambiguous; the example must commit to one track).
///
/// `feature` is always the first path component under `examples_dir`. `dir`
/// is the remainder of the path joined with '/' (e.g. for
/// `examples/industrial/sha2/sha256-digest/hirusttest.toml` we get
/// `feature = "industrial"`, `dir = "sha2/sha256-digest"`).
pub fn find_examples(examples_dir: &Path) -> Result<Vec<Example>> {
    let mut result = Vec::new();
    let walker = WalkDir::new(examples_dir).into_iter().filter_entry(|e| {
        if !e.file_type().is_dir() {
            return true;
        }
        // Skip a directory entirely (and its descendants) if it has the
        // .no-hirusttest marker. This is the "屏蔽" hook.
        if e.path().join(SKIP_MARKER).exists() {
            return false;
        }
        // Never descend into a `.hirusttest/` subtree — its contents are
        // auxiliary files for the parent example, not nested examples. (The
        // parent example is detected at the parent dir level.)
        if e.file_name() == DIR_TRACK_DIRNAME {
            return false;
        }
        true
    });
    for entry in walker {
        let entry = entry.with_context(|| format!("walking {}", examples_dir.display()))?;
        if !entry.file_type().is_dir() {
            continue;
        }
        let dir = entry.path();
        // The examples_dir itself is the walker root — it has no signal files
        // and no first-level component relative to itself. Skip explicitly.
        if dir == examples_dir {
            continue;
        }

        let single_file = dir.join(SINGLE_FILE_TRACK_NAME);
        let dir_track_root = dir.join(DIR_TRACK_DIRNAME);
        let dir_track_config = dir_track_root.join(DIR_TRACK_CONFIG_NAME);

        let single_file_exists = single_file.exists();
        let dir_track_config_exists = dir_track_config.exists();
        let dir_track_root_exists = dir_track_root.exists();

        // Ambiguity: both tracks signal at once. Per detailed-design §一 边界 3,
        // this must be a hard error — silent precedence would let a dropped
        // hirusttest.toml mask a directory-track config, or vice versa.
        if single_file_exists && dir_track_root_exists {
            return Err(anyhow!(
                "ambiguous schema at {}: both `{}` and `{}/` exist; an example must commit to exactly one track",
                dir.display(),
                SINGLE_FILE_TRACK_NAME,
                DIR_TRACK_DIRNAME,
            ));
        }

        let (config_path, schema_kind, hirusttest_dir) = if dir_track_config_exists {
            (
                dir_track_config.clone(),
                SchemaKind::Directory,
                Some(dir_track_root.clone()),
            )
        } else if single_file_exists {
            (single_file.clone(), SchemaKind::SingleFile, None)
        } else {
            // No signal — not an example. (A `.hirusttest/` directory without
            // `config.toml` inside is also treated as no signal: malformed
            // setup, but not our place to error here; the user just won't see
            // the example registered.)
            continue;
        };

        let ts_text = fs::read_to_string(&config_path)
            .with_context(|| format!("reading {}", config_path.display()))?;
        let ts: HirusttestToml = toml::from_str(&ts_text)
            .with_context(|| format!("parsing {}", config_path.display()))?;

        // entries must be non-empty. Per detailed-design.md §一: `entries =
        // ["fn_name_1", ...]` is documented as "必填，零参 pub fn 名列表";
        // `[runnable.<entry>]` sub-tables (line ~78) only *extend* an entry
        // that already appears in `entries`, they never create one. An empty
        // `entries = []` therefore produces zero tasks silently, which is a
        // config bug per design §七 ("配置 bug, 不该静默"). Fail at discover.
        if ts.entries.is_empty() {
            return Err(anyhow!(
                "{}: `entries` must be non-empty (an example must register at least one pub fn name)",
                config_path.display()
            ));
        }

        // Canonicalize example.root so we can test containment in absolute,
        // symlink-resolved form. dir itself comes from the walker and may
        // contain unresolved symlinks; without canonicalize, starts_with
        // would compare apples to oranges with the canonicalized target_dir.
        let root_canon = dir.canonicalize().with_context(|| {
            format!("canonicalizing example root {}", dir.display())
        })?;

        let target_rel = ts.target_path.as_deref().unwrap_or(".");
        let target_dir = dir.join(target_rel).canonicalize().with_context(|| {
            format!(
                "canonicalizing target_path '{}' under {}",
                target_rel,
                dir.display()
            )
        })?;

        // target_path must resolve to a subdirectory of example.root. Without
        // this check, `target_path = "../../escape"` escapes the isolated copy
        // and exec stage only catches it via strip_prefix as UNKNOWN — design
        // (detailed-design.md §三, architecture.md §四) requires Err at
        // discover.
        if !target_dir.starts_with(&root_canon) {
            return Err(anyhow!(
                "{}: target_path '{}' resolves to {} which is not under example root {}",
                config_path.display(),
                target_rel,
                target_dir.display(),
                root_canon.display(),
            ));
        }

        let cargo_path = target_dir.join("Cargo.toml");
        let cargo_text = fs::read_to_string(&cargo_path)
            .with_context(|| format!("reading {}", cargo_path.display()))?;
        let cargo: CargoToml = toml::from_str(&cargo_text)
            .with_context(|| format!("parsing {}", cargo_path.display()))?;
        if cargo.package.name.is_empty() {
            return Err(anyhow!(
                "[package].name is empty in {}",
                cargo_path.display()
            ));
        }

        let rel = dir.strip_prefix(examples_dir).unwrap();
        let mut comps = rel.components();
        let feature = comps
            .next()
            .ok_or_else(|| anyhow!("example {} has no feature dir", dir.display()))?
            .as_os_str()
            .to_string_lossy()
            .to_string();
        let rest: Vec<String> = comps
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        if rest.is_empty() {
            return Err(anyhow!(
                "example {} has no inner dir under feature '{}'",
                dir.display(),
                feature
            ));
        }
        let dir_name = rest.join("/");

        // Build entry_args map from any [runnable.<entry>] tables. We render
        // inputs[0] as a comma-separated list of Rust literal expressions so
        // every harness template that calls `{{ entry_fn }}({{ entry_args }})`
        // compiles regardless of whether the fn takes args or not.
        //
        // Type matrix per detailed-design.md §一 ("类型支持矩阵"):
        //   integers (i*/u*) → `<n>{type}` chosen by signature (we don't see
        //     the fn signature here; emit untyped numeric literals — Rust
        //     does fn-arg inference, so `add_two(3, 4)` works for any
        //     concrete integer type the fn declares). Negative numbers render
        //     as `-N` (toml integer is i64-domain).
        //   bool → `true` / `false`
        //   v1 disallows other types; we still accept them silently so a
        //     future runnable test with a struct argument can stop being a
        //     hard-error before its consumer lands.
        let mut entry_args = std::collections::HashMap::new();
        for entry_name in &ts.entries {
            if let Some(spec) = ts.runnable.get(entry_name) {
                let row = spec.inputs.first().ok_or_else(|| {
                    anyhow!(
                        "{}: [runnable.{}].inputs must have at least one row",
                        config_path.display(),
                        entry_name
                    )
                })?;
                let args = render_runnable_row_as_rust_args(row).with_context(|| {
                    format!(
                        "{}: rendering [runnable.{}].inputs[0] as Rust args",
                        config_path.display(),
                        entry_name
                    )
                })?;
                entry_args.insert(entry_name.clone(), args);
            }
        }

        result.push(Example {
            feature,
            dir: dir_name,
            root: dir.to_path_buf(),
            target_path: target_dir,
            crate_name: cargo.package.name,
            entries: ts.entries,
            entry_args,
            schema_kind,
            hirusttest_dir,
            env: ts.env,
        });
    }
    result.sort_by(|a, b| {
        (a.feature.as_str(), a.dir.as_str()).cmp(&(b.feature.as_str(), b.dir.as_str()))
    });
    Ok(result)
}

/// Render one `inputs` row (a TOML array of scalars) as a Rust argument
/// expression string suitable for splicing into `fn_name({{ entry_args }})`.
/// Supports the v1 runnable type matrix (i*/u*/bool); rejects strings /
/// tables / nested arrays (Tera output would fail to compile and cargo would
/// expose it as a hard error — preferable to silent skip).
fn render_runnable_row_as_rust_args(row: &toml::Value) -> Result<String> {
    let items = row
        .as_array()
        .ok_or_else(|| anyhow!("inputs row must be a TOML array; got {:?}", row.type_str()))?;
    let mut parts = Vec::with_capacity(items.len());
    for v in items {
        let rendered = match v {
            toml::Value::Integer(n) => n.to_string(),
            toml::Value::Boolean(b) => b.to_string(),
            other => {
                return Err(anyhow!(
                    "inputs element type `{}` is not in the v1 runnable type matrix (i*/u*/bool)",
                    other.type_str()
                ));
            }
        };
        parts.push(rendered);
    }
    Ok(parts.join(", "))
}

/// Where to render the harness inside the working copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryMode {
    /// Default. Render harness to `src/bin/__ts_harness.rs` — example lib stays
    /// the crate's lib target, harness is a binary alongside it.
    Bin,
    /// Render harness to `src/lib.rs`, after first renaming the original
    /// `src/lib.rs` to `src/__ts_inner.rs`. The harness becomes the crate's
    /// lib target; the original lib's contents are pulled in as `mod __ts_inner`
    /// so all public items remain reachable as `<crate>::*`. Used by tools
    /// (e.g. Creusot) that require the *top-level lib crate itself* to import
    /// tool-specific prelude.
    Lib,
}

impl Default for EntryMode {
    fn default() -> Self { EntryMode::Bin }
}

/// A discovered tool integration: tools/<name>/{tool.toml + harness.rs.tera}.
pub struct Tool {
    pub name: String,
    pub command: Vec<String>,
    /// Per-tool timeout in seconds. After this duration the subprocess is
    /// SIGKILL'd and the task is marked FAILED with `timed_out: true` (see §7.5).
    pub timeout_secs: u64,
    pub harness_template: String,
    /// Lines to append to the working-copy `Cargo.toml`'s `[dependencies]` section
    /// before invoking the tool. Each entry is a TOML dependency line such as
    /// `creusot-std = "0.11.0"`. Used by tools (e.g. Creusot) that require the
    /// example crate to depend on a tool-specific support crate.
    pub extra_cargo_deps: Vec<String>,
    /// Where to render the harness — see `EntryMode`. Default `Bin`.
    pub entry_mode: EntryMode,
    /// argv to capture this tool's version string (run once at run start).
    /// Empty = skip version capture.
    pub version_command: Vec<String>,
}

#[derive(Deserialize)]
struct ToolToml {
    command: Vec<String>,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
    #[serde(default)]
    extra_cargo_deps: Vec<String>,
    #[serde(default)]
    entry_mode: EntryMode,
    #[serde(default)]
    version_command: Vec<String>,
}

fn default_timeout() -> u64 {
    300
}

/// Expand `${VAR}` references in `s` against the runner's process environment.
/// Variable names must match `[A-Z_][A-Z0-9_]*`. Missing variables expand to
/// the empty string. Non-matching `$...` sequences are left untouched. Users
/// supply their environment by `source .env` (or equivalent) before invoking
/// the runner — `.env.example` documents the expected variable names.
fn expand_env(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        if chars.peek() != Some(&'{') {
            out.push('$');
            continue;
        }
        chars.next(); // consume '{'
        let mut name = String::new();
        let mut closed = false;
        while let Some(&nc) = chars.peek() {
            if nc == '}' {
                chars.next();
                closed = true;
                break;
            }
            name.push(nc);
            chars.next();
        }
        let valid = !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_');
        if closed && valid {
            out.push_str(&std::env::var(&name).unwrap_or_default());
        } else {
            // Not a recognized var-ref — emit literally, including any chars consumed.
            out.push('$');
            out.push('{');
            out.push_str(&name);
            if closed {
                out.push('}');
            }
        }
    }
    out
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
        if parsed.command.is_empty() {
            return Err(anyhow!(
                "tool {} has empty `command` in {}",
                dir.display(),
                tool_toml_path.display()
            ));
        }
        // Validate `extra_cargo_deps` at discover time. Each entry is a single
        // TOML key=value fragment (e.g. `creusot-std = "0.11.0"`); exec stage
        // splices it into the working-copy Cargo.toml via toml_edit. Per
        // detailed-design.md §七 ("Schema 解析失败 ... → discover 阶段 panic"),
        // bad TOML in tool.toml is a config bug and must surface before any
        // task is launched, not as a per-task UNKNOWN later.
        for dep_line in &parsed.extra_cargo_deps {
            let parsed_dep: toml_edit::DocumentMut = dep_line
                .parse()
                .with_context(|| {
                    format!(
                        "{}: extra_cargo_deps entry `{}` is not valid TOML",
                        tool_toml_path.display(),
                        dep_line
                    )
                })?;
            // A well-formed entry contributes at least one top-level key
            // (the dependency name). An entry that parses but contributes no
            // keys (e.g. an empty string, a stray comment) is also a config
            // bug — patch_cargo_deps would silently do nothing.
            if parsed_dep.as_table().iter().next().is_none() {
                return Err(anyhow!(
                    "{}: extra_cargo_deps entry `{}` declares no dependency key",
                    tool_toml_path.display(),
                    dep_line
                ));
            }
        }
        let template = fs::read_to_string(&harness_path)
            .with_context(|| format!("reading {}", harness_path.display()))?;

        let name = dir
            .file_name()
            .ok_or_else(|| anyhow!("bad tool dir {}", dir.display()))?
            .to_string_lossy()
            .to_string();

        // Expand ${VAR} in command/version_command against the runner process
        // env (sourced from .env by the user before launch). Keeps argv as an
        // array; no shell wrapping is required for variable substitution.
        let command: Vec<String> = parsed.command.into_iter().map(|s| expand_env(&s)).collect();
        let version_command: Vec<String> = parsed
            .version_command
            .into_iter()
            .map(|s| expand_env(&s))
            .collect();

        result.push(Tool {
            name,
            command,
            timeout_secs: parsed.timeout_secs,
            harness_template: template,
            extra_cargo_deps: parsed.extra_cargo_deps,
            entry_mode: parsed.entry_mode,
            version_command,
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}
