//! Collect host info & format ISO 8601 UTC timestamps.
//!
//! Used by `report.rs` to populate `results.json`'s `host` and time fields.
//! Each field tries to read its source; on failure, the field is `None` —
//! never blocks the run.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HostInfo {
    pub hostname: Option<String>,
    /// "macos" / "linux" / "windows" / ...  (from `std::env::consts::OS`)
    pub os: String,
    /// "aarch64" / "x86_64" / ...  (from `std::env::consts::ARCH`)
    pub arch: String,
    /// e.g. macOS `uname -r` => "Darwin 25.4.0"-style; just the kernel release string.
    pub kernel: Option<String>,
    /// CPU brand string (macOS `sysctl machdep.cpu.brand_string`; linux `/proc/cpuinfo`).
    pub cpu_brand: Option<String>,
    /// Total physical memory in MB.
    pub total_mem_mb: Option<u64>,
    /// As reported by the runtime (`std::thread::available_parallelism()`).
    pub num_cpus: Option<usize>,
}

pub fn collect() -> HostInfo {
    // P41: hostname intentionally NOT collected (was reviewer A finding for
    // anonymization in published artifacts; the actual hostname carries no
    // reproducibility value — OS / arch / kernel / CPU brand / mem / cpus
    // suffice to characterize the host class).
    HostInfo {
        hostname: None,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        kernel: cmd_oneline("uname", &["-r"]),
        cpu_brand: cpu_brand(),
        total_mem_mb: total_mem_mb(),
        num_cpus: std::thread::available_parallelism().ok().map(|n| n.get()),
    }
}

fn cmd_oneline(prog: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(prog).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn cpu_brand() -> Option<String> {
    if cfg!(target_os = "macos") {
        cmd_oneline("sysctl", &["-n", "machdep.cpu.brand_string"])
    } else if cfg!(target_os = "linux") {
        let txt = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        txt.lines()
            .find_map(|l| l.strip_prefix("model name"))
            .and_then(|rest| rest.split_once(':').map(|(_, v)| v.trim().to_string()))
    } else {
        None
    }
}

fn total_mem_mb() -> Option<u64> {
    if cfg!(target_os = "macos") {
        let bytes: u64 = cmd_oneline("sysctl", &["-n", "hw.memsize"])?.parse().ok()?;
        Some(bytes / 1_048_576)
    } else if cfg!(target_os = "linux") {
        let txt = std::fs::read_to_string("/proc/meminfo").ok()?;
        for line in txt.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
                return Some(kb / 1024);
            }
        }
        None
    } else {
        None
    }
}

/// Format a `SystemTime` as an ISO 8601 UTC string ("2026-05-07T16:02:11Z").
/// Uses `libc::gmtime_r`; safe (writes to local struct, no shared state).
pub fn iso8601_utc(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let secs_t = secs as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::gmtime_r(&secs_t, &mut tm) };
    if r.is_null() {
        return format!("@{}s", secs);
    }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

/// Run a tool's `version_command` (full argv) with a short timeout in env;
/// capture stdout (preferred) or stderr; trim & truncate to 500 bytes.
/// `None` if the command isn't configured or fails to launch.
pub fn capture_version(version_command: &[String]) -> Option<String> {
    if version_command.is_empty() {
        return None;
    }
    let out = Command::new(&version_command[0])
        .args(&version_command[1..])
        .output()
        .ok()?;
    let raw = if !out.stdout.is_empty() {
        &out.stdout
    } else {
        &out.stderr
    };
    let mut s = String::from_utf8_lossy(raw).trim().to_string();
    if s.len() > 500 {
        s.truncate(500);
        s.push_str("...");
    }
    if s.is_empty() { None } else { Some(s) }
}
