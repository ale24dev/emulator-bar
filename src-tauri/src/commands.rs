//! Tauri commands exposed to the frontend. All are async so process
//! execution never blocks the main (UI) thread.

use crate::devices::{
    Device, EmulatorProvider, android::AndroidProvider, ios::IosProvider, tools::ToolPaths,
};

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

#[tauri::command]
pub async fn list_devices(state: tauri::State<'_, Providers>) -> Result<Vec<Device>, String> {
    state.list_all().await
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
}
