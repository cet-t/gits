use std::process::{Command, Stdio};

use crate::error::Error;

pub fn output(args: &[&str]) -> Result<String, Error> {
    let result = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(Error::GitNotFound)?;

    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).into_owned())
    } else {
        Err(Error::GitFailed {
            code: result.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&result.stderr).trim().to_owned(),
        })
    }
}

pub fn exec(args: &[String]) -> Result<std::process::ExitStatus, Error> {
    Command::new("git")
        .args(args)
        .status()
        .map_err(Error::GitNotFound)
}

pub fn log_lines() -> Result<Vec<String>, Error> {
    let raw = output(&["log", "--oneline", "--decorate"])?;
    Ok(raw.lines().filter(|l| !l.is_empty()).map(str::to_owned).collect())
}

/// Extract the commit hash (first whitespace-delimited token) from a log line.
pub fn hash_from_line(line: &str) -> &str {
    line.split_whitespace().next().unwrap_or(line)
}

pub fn branches() -> Result<Vec<String>, Error> {
    let raw = output(&["branch", "--format=%(refname:short)"])?;
    let current = current_branch().unwrap_or_default();
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|b| !b.is_empty() && *b != current.trim())
        .map(str::to_owned)
        .collect())
}

fn current_branch() -> Result<String, Error> {
    output(&["rev-parse", "--abbrev-ref", "HEAD"]).map(|s| s.trim().to_owned())
}
