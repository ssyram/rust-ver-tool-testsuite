use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct TaskResult {
    /// Full ID: "<feature>/<dir>/<entry-fn>"
    pub entry_id: String,
    pub tool: String,
    /// "SUCCESS" or "FAILED"
    pub status: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    /// Relative path (from run_dir) to raw stdout; None if runner errored before subprocess.
    pub raw_stdout: Option<String>,
    pub raw_stderr: Option<String>,
}

#[derive(Serialize)]
struct ResultsFile<'a> {
    run_id: &'a str,
    results: &'a [TaskResult],
}

/// Write run_dir/results.json — machine-readable summary.
pub fn write_results_json(run_dir: &Path, run_id: &str, results: &[TaskResult]) -> Result<()> {
    let file = ResultsFile { run_id, results };
    let json = serde_json::to_string_pretty(&file).context("serializing results.json")?;
    let path = run_dir.join("results.json");
    std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Write run_dir/report.md — human-readable matrix grouped by feature.
pub fn write_report_md(run_dir: &Path, run_id: &str, results: &[TaskResult]) -> Result<()> {
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
    md.push_str(&format!("# Run {}\n\n", run_id));

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
    let f = results.len() - s;
    md.push_str(&format!(
        "---\nTotal: {} succeeded / {} failed / {} total\n",
        s, f, results.len()
    ));

    let path = run_dir.join("report.md");
    std::fs::write(&path, md).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}
