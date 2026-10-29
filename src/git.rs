use std::process::{Command, Stdio};

use crate::ansi;
use crate::error::{GitsError, GitsResult};

pub fn output(args: &[&str]) -> GitsResult<String> {
    let result = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(GitsError::GitNotFound)?;

    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).into_owned())
    } else {
        Err(GitsError::GitFailed {
            code: result.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&result.stderr).trim().to_owned(),
        })
    }
}

pub fn execute(args: &[String]) -> GitsResult<std::process::ExitStatus> {
    Command::new("git")
        .args(args)
        .status()
        .map_err(GitsError::GitNotFound)
}

/// Log lines colored the same way as `git log` (hash, HEAD, branches, tags).
pub fn log_lines() -> GitsResult<Vec<String>> {
    let raw = output(&["log", "--oneline", "--decorate", "--color=always"])?;
    Ok(raw
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Extract the commit hash (first whitespace-delimited token) from a log line.
pub fn hash_from_line(line: &str) -> String {
    let plain = ansi::strip(line);
    plain
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// Color branch names like local branches in `git log --decorate`,
/// honoring `color.decorate.branch`.
pub fn colored_branches(branches: &[String]) -> Vec<String> {
    let color = output(&[
        "config",
        "--get-color",
        "color.decorate.branch",
        "bold green",
    ])
    .unwrap_or_default();
    let reset = output(&["config", "--get-color", "", "reset"]).unwrap_or_default();
    branches
        .iter()
        .map(|b| format!("{color}{b}{reset}"))
        .collect()
}

pub fn branches() -> Result<Vec<String>, GitsError> {
    let raw = output(&["branch", "--format=%(refname:short)"])?;
    let current = current_branch().unwrap_or_default();
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|&b| !b.is_empty() && b != current.trim())
        .map(str::to_owned)
        .collect())
}

fn current_branch() -> GitsResult<String> {
    output(&["rev-parse", "--abbrev-ref", "HEAD"]).map(|s| s.trim().to_owned())
}
