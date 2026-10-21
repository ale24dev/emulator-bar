//! Device model and the provider abstraction shared by Android and iOS.

pub mod actions;
pub mod android;
pub mod ios;
pub mod process;
pub mod tools;

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Android,
    Ios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceState {
    Booted,
    Booting,
    ShuttingDown,
    Shutdown,
    Unknown,
}

/// A device as presented to the UI, regardless of platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// iOS: simulator UDID. Android: AVD name.
    pub id: String,
    pub name: String,
    pub platform: Platform,
    pub state: DeviceState,
    pub os_version: Option<String>,
    /// Android only: the adb serial (e.g. `emulator-5554`) while running.
    pub serial: Option<String>,
}

/// Operations every platform backend supports. `id` is [`Device::id`].
#[allow(async_fn_in_trait)]
pub trait EmulatorProvider {
    async fn list(&self) -> Result<Vec<Device>, String>;
    async fn boot(&self, id: &str) -> Result<(), String>;
    async fn shutdown(&self, id: &str) -> Result<(), String>;
    async fn open_url(&self, id: &str, url: &str) -> Result<(), String>;
    /// Installs an `.apk` (Android) or `.app` bundle (iOS).
    async fn install(&self, id: &str, path: &str) -> Result<(), String>;
    /// Saves a PNG screenshot at `dest`.
    async fn screenshot(&self, id: &str, dest: &Path) -> Result<(), String>;
    /// Wipes user data. The device must be shut down.
    async fn erase(&self, id: &str) -> Result<(), String>;
}
