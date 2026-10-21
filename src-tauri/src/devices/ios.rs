//! iOS simulators via `xcrun simctl`.

use super::{Device, DeviceState, EmulatorProvider, Platform, actions, process, tools::ToolPaths};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Apps that display booted simulators, in preference order:
/// Simulator.app (Xcode <= 26) and DeviceHub.app (Xcode 27+).
const SIMULATOR_VIEWER_BUNDLE_IDS: [&str; 2] =
    ["com.apple.iphonesimulator", "com.apple.dt.Devices"];

#[derive(Deserialize)]
struct SimctlList {
    devices: HashMap<String, Vec<SimctlDevice>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SimctlDevice {
    udid: String,
    name: String,
    state: String,
    #[serde(default)]
    is_available: bool,
}

/// Parses `xcrun simctl list devices --json`, keeping only available iOS
/// simulators, sorted by OS version (newest first) then name.
pub fn parse_simctl_devices(json: &str) -> Result<Vec<Device>, String> {
    let list: SimctlList =
        serde_json::from_str(json).map_err(|e| format!("Unexpected simctl output: {e}"))?;

    let mut devices: Vec<(Vec<u32>, Device)> = list
        .devices
        .into_iter()
        .filter_map(|(runtime, sims)| parse_runtime_version(&runtime).map(|v| (v, sims)))
        .flat_map(|(version, sims)| {
            let key = version_key(&version);
            sims.into_iter().filter(|s| s.is_available).map(move |s| {
                let device = Device {
                    id: s.udid,
                    name: s.name,
                    platform: Platform::Ios,
                    state: parse_sim_state(&s.state),
                    os_version: Some(version.clone()),
                    serial: None,
                };
                (key.clone(), device)
            })
        })
        .collect();

    devices.sort_by(|(va, a), (vb, b)| {
        vb.cmp(va)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(devices.into_iter().map(|(_, d)| d).collect())
}

fn version_key(version: &str) -> Vec<u32> {
    version
        .trim_start_matches("iOS ")
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect()
}

/// `com.apple.CoreSimulator.SimRuntime.iOS-27-0` -> `Some("iOS 27.0")`.
/// Returns `None` for non-iOS runtimes (watchOS, tvOS, visionOS).
pub fn parse_runtime_version(runtime: &str) -> Option<String> {
    let suffix = runtime.rsplit('.').next()?.strip_prefix("iOS-")?;
    Some(format!("iOS {}", suffix.replace('-', ".")))
}

pub fn parse_sim_state(state: &str) -> DeviceState {
    match state {
        "Booted" => DeviceState::Booted,
        "Booting" => DeviceState::Booting,
        "Shutting Down" => DeviceState::ShuttingDown,
        "Shutdown" => DeviceState::Shutdown,
        _ => DeviceState::Unknown,
    }
}

pub struct IosProvider {
    tools: ToolPaths,
}

impl IosProvider {
    pub fn new(tools: ToolPaths) -> Self {
        Self { tools }
    }

    async fn simctl(&self, args: &[&str]) -> Result<String, String> {
        let mut full = vec!["simctl"];
        full.extend_from_slice(args);
        process::run(&self.tools.xcrun, &full).await
    }
}

impl IosProvider {
    async fn state_of(&self, udid: &str) -> Result<DeviceState, String> {
        self.list()
            .await?
            .into_iter()
            .find(|d| d.id == udid)
            .map(|d| d.state)
            .ok_or_else(|| format!("Simulator {udid} not found"))
    }
}

impl EmulatorProvider for IosProvider {
    async fn list(&self) -> Result<Vec<Device>, String> {
        let json = self
            .simctl(&["list", "devices", "available", "--json"])
            .await?;
        parse_simctl_devices(&json)
    }

    async fn boot(&self, udid: &str) -> Result<(), String> {
        match self.simctl(&["boot", udid]).await {
            Ok(_) => {}
            // Booting an already booted simulator is not an error for the user.
            Err(e) if e.contains("current state: Booted") => {}
            Err(e) => return Err(e),
        }
        // `simctl boot` is headless; bring up the device window. Xcode 27 replaced
        // Simulator.app with DeviceHub.app, so try both bundle ids. The device is
        // already booted at this point, so a missing viewer is not an error.
        for bundle_id in SIMULATOR_VIEWER_BUNDLE_IDS {
            if process::run(Path::new("/usr/bin/open"), &["-b", bundle_id])
                .await
                .is_ok()
            {
                break;
            }
        }
        Ok(())
    }

    async fn shutdown(&self, udid: &str) -> Result<(), String> {
        match self.simctl(&["shutdown", udid]).await {
            Err(e) if !e.contains("current state: Shutdown") => Err(e),
            _ => Ok(()),
        }
    }

    async fn open_url(&self, udid: &str, url: &str) -> Result<(), String> {
        let url = actions::validate_url(url)?;
        self.simctl(&["openurl", udid, url]).await.map(drop)
    }

    async fn install(&self, udid: &str, path: &str) -> Result<(), String> {
        let app = actions::validate_package(path, "app")?;
        let app = app.to_string_lossy();
        self.simctl(&["install", udid, &app]).await.map(drop)
    }

    async fn screenshot(&self, udid: &str, dest: &Path) -> Result<(), String> {
        let dest = dest.to_string_lossy();
        self.simctl(&["io", udid, "screenshot", &dest])
            .await
            .map(drop)
    }

    async fn erase(&self, udid: &str) -> Result<(), String> {
        if self.state_of(udid).await? != DeviceState::Shutdown {
            return Err("Shut down the simulator before erasing it".into());
        }
        self.simctl(&["erase", udid]).await.map(drop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/simctl_devices.json");

    #[test]
    fn keeps_only_available_ios_simulators() {
        let devices = parse_simctl_devices(FIXTURE).unwrap();
        let names: Vec<&str> = devices.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["iPhone 17e", "iPhone 18 Pro", "iPhone 18 Pro Max"]
        );
    }

    #[test]
    fn maps_fields_into_unified_device() {
        let devices = parse_simctl_devices(FIXTURE).unwrap();
        let booted = devices
            .iter()
            .find(|d| d.name == "iPhone 18 Pro Max")
            .unwrap();
        assert_eq!(booted.id, "B74A192C-852D-496E-BC63-AECB12F13402");
        assert_eq!(booted.platform, Platform::Ios);
        assert_eq!(booted.state, DeviceState::Booted);
        assert_eq!(booted.os_version.as_deref(), Some("iOS 27.0"));
        assert_eq!(booted.serial, None);
    }

    #[test]
    fn rejects_invalid_json_with_readable_error() {
        let err = parse_simctl_devices("not json").unwrap_err();
        assert!(err.contains("simctl"), "{err}");
    }

    #[test]
    fn parses_runtime_versions() {
        assert_eq!(
            parse_runtime_version("com.apple.CoreSimulator.SimRuntime.iOS-27-0").as_deref(),
            Some("iOS 27.0")
        );
        assert_eq!(
            parse_runtime_version("com.apple.CoreSimulator.SimRuntime.iOS-18-6-1").as_deref(),
            Some("iOS 18.6.1")
        );
        assert_eq!(
            parse_runtime_version("com.apple.CoreSimulator.SimRuntime.watchOS-12-0"),
            None
        );
    }

    #[test]
    fn parses_simulator_states() {
        assert_eq!(parse_sim_state("Booted"), DeviceState::Booted);
        assert_eq!(parse_sim_state("Booting"), DeviceState::Booting);
        assert_eq!(parse_sim_state("Shutting Down"), DeviceState::ShuttingDown);
        assert_eq!(parse_sim_state("Shutdown"), DeviceState::Shutdown);
        assert_eq!(parse_sim_state("Creating"), DeviceState::Unknown);
    }
}
