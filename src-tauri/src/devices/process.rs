//! Async process execution helpers. Never blocks the UI thread.

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Runs a program to completion and returns its stdout, or a readable error
/// containing stderr when it exits unsuccessfully.
pub async fn run(program: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Failed to run {}: {e}", program.display()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(failure_message(program, args, &output))
    }
}

fn failure_message(program: &Path, args: &[&str], output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    let name = program
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("command");
    format!("{name} {} failed: {detail}", args.join(" "))
}
