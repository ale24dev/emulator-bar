//! Tauri commands exposed to the frontend. All are async so process
//! execution never blocks the main (UI) thread.

use crate::devices::{
    Device, DeviceState, EmulatorProvider, Platform, actions, android::AndroidProvider,
    ios::IosProvider, tools::ToolPaths,
};
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

pub struct Providers {
    pub android: AndroidProvider,
    pub ios: IosProvider,
}

impl Providers {
    pub fn new(tools: ToolPaths) -> Self {
        Self {
            android: AndroidProvider::new(tools.clone()),
            ios: IosProvider::new(tools),
        }
    }

    /// Lists both platforms concurrently. A failing platform yields an error
    /// only when both fail, so one missing toolchain does not hide the other.
    pub async fn list_all(&self) -> Result<Vec<Device>, String> {
        let (ios, android) = tokio::join!(self.ios.list(), self.android.list());
        match (ios, android) {
            (Err(a), Err(b)) => Err(format!("{a}\n{b}")),
            (ios, android) => {
                let mut all = ios.unwrap_or_default();
                all.extend(android.unwrap_or_default());
                Ok(all)
            }
        }
    }
}

/// Dispatches one provider method by platform: `dispatch!(self, platform, boot(id))`.
macro_rules! dispatch {
    ($providers:expr, $platform:expr, $method:ident($($arg:expr),*)) => {
        match $platform {
            Platform::Android => $providers.android.$method($($arg),*).await,
            Platform::Ios => $providers.ios.$method($($arg),*).await,
        }
    };
}

/// Outcome of a bulk action, so the UI can report partial failures.
#[derive(Debug, Serialize)]
pub struct BulkResult {
    pub succeeded: usize,
    pub failed: Vec<String>,
}

impl BulkResult {
    fn record(&mut self, device: &Device, result: Result<(), String>) {
        match result {
            Ok(()) => self.succeeded += 1,
            Err(e) => self.failed.push(format!("{}: {e}", device.name)),
        }
    }
}

fn desktop_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Desktop"))
        .ok_or_else(|| "HOME is not set".to_string())
}

#[tauri::command]
pub async fn list_devices(state: State<'_, Providers>) -> Result<Vec<Device>, String> {
    state.list_all().await
}

#[tauri::command]
pub async fn boot_device(
    state: State<'_, Providers>,
    platform: Platform,
    id: String,
) -> Result<(), String> {
    dispatch!(state, platform, boot(&id))
}

#[tauri::command]
pub async fn shutdown_device(
    state: State<'_, Providers>,
    platform: Platform,
    id: String,
) -> Result<(), String> {
    dispatch!(state, platform, shutdown(&id))
}

#[tauri::command]
pub async fn open_url(
    state: State<'_, Providers>,
    platform: Platform,
    id: String,
    url: String,
) -> Result<(), String> {
    dispatch!(state, platform, open_url(&id, &url))
}

#[tauri::command]
pub async fn install_app(
    state: State<'_, Providers>,
    platform: Platform,
    id: String,
    path: String,
) -> Result<(), String> {
    dispatch!(state, platform, install(&id, &path))
}

/// Saves a screenshot to `~/Desktop/<name>-<timestamp>.png` and returns its path.
#[tauri::command]
pub async fn take_screenshot(
    state: State<'_, Providers>,
    platform: Platform,
    id: String,
    name: String,
) -> Result<String, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let dest = actions::screenshot_path(&desktop_dir()?, &name, &timestamp);
    dispatch!(state, platform, screenshot(&id, &dest))?;
    Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn erase_device(
    state: State<'_, Providers>,
    platform: Platform,
    id: String,
) -> Result<(), String> {
    dispatch!(state, platform, erase(&id))
}

#[tauri::command]
pub async fn shutdown_all(state: State<'_, Providers>) -> Result<BulkResult, String> {
    let mut result = BulkResult {
        succeeded: 0,
        failed: Vec::new(),
    };
    for device in state.list_all().await? {
        if device.state == DeviceState::Booted {
            let outcome = dispatch!(state, device.platform, shutdown(&device.id));
            result.record(&device, outcome);
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn open_url_all_booted(
    state: State<'_, Providers>,
    url: String,
) -> Result<BulkResult, String> {
    let url = actions::validate_url(&url)?.to_string();
    let mut result = BulkResult {
        succeeded: 0,
        failed: Vec::new(),
    };
    for device in state.list_all().await? {
        if device.state == DeviceState::Booted {
            let outcome = dispatch!(state, device.platform, open_url(&device.id, &url));
            result.record(&device, outcome);
        }
    }
    if result.succeeded == 0 && result.failed.is_empty() {
        return Err("No booted devices".into());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test against the real toolchains on this machine:
    /// `cargo test -- --ignored --nocapture`.
    #[tokio::test]
    #[ignore = "requires local Xcode / Android SDK"]
    async fn lists_real_devices() {
        let providers = Providers::new(ToolPaths::from_env());
        let devices = providers.list_all().await.expect("listing failed");
        for d in &devices {
            println!(
                "{:?}\t{:<22}\t{:?}\t{}\t{}\t{}",
                d.platform,
                d.name,
                d.state,
                d.os_version.as_deref().unwrap_or("-"),
                d.serial.as_deref().unwrap_or("-"),
                d.id
            );
        }
        assert!(!devices.is_empty());
    }

    /// Screenshots every booted device into the temp dir (non-destructive).
    #[tokio::test]
    #[ignore = "requires a booted emulator or simulator"]
    async fn screenshots_booted_devices() {
        let providers = Providers::new(ToolPaths::from_env());
        let booted: Vec<Device> = providers
            .list_all()
            .await
            .unwrap()
            .into_iter()
            .filter(|d| d.state == DeviceState::Booted)
            .collect();
        for d in booted {
            let dest = actions::screenshot_path(&std::env::temp_dir(), &d.name, "smoke");
            dispatch!(providers, d.platform, screenshot(&d.id, &dest)).expect("screenshot failed");
            let size = std::fs::metadata(&dest).unwrap().len();
            println!("{} -> {} ({size} bytes)", d.name, dest.display());
            assert!(size > 0);
            std::fs::remove_file(dest).unwrap();
        }
    }
}
