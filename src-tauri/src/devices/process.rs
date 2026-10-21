//! Async process execution helpers. Never blocks the UI thread.

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Runs a program to completion and returns its stdout, or a readable error
/// containing stderr when it exits unsuccessfully.
pub async fn run(program: &Path, args: &[&str]) -> Result<String, String> {
    let bytes = run_bytes(program, args).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Like [`run`] but returns raw stdout bytes (e.g. for screenshots).
pub async fn run_bytes(program: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Failed to run {}: {e}", program.display()))?;

    if output.status.success() {
        Ok(output.stdout)
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

/// Starts a long-running program without waiting for it (e.g. the emulator).
/// The child is not killed when the handle drops; tokio reaps it on exit.
pub fn spawn_detached(program: &Path, args: &[&str]) -> Result<(), String> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(drop)
        .map_err(|e| format!("Failed to start {}: {e}", program.display()))
}
