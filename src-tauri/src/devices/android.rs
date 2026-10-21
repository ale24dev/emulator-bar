//! Android emulators via `emulator` and `adb`.

use super::{Device, DeviceState, EmulatorProvider, Platform, process, tools::ToolPaths};
use std::collections::HashMap;

/// An adb-visible emulator instance (`emulator-5554  device`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdbEmulator {
    pub serial: String,
    pub state: DeviceState,
}

/// Metadata read from `<avd_home>/<name>.avd/config.ini`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AvdConfig {
    pub display_name: Option<String>,
    pub os_version: Option<String>,
}

/// Parses `emulator -list-avds`, skipping log/noise lines.
pub fn parse_avd_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        // AVD names only contain [A-Za-z0-9._-]; anything else is emulator logging.
        .filter(|l| {
            !l.is_empty()
                && l.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        })
        .map(String::from)
        .collect()
}

/// Parses `adb devices`, keeping only emulator serials (physical devices are ignored).
pub fn parse_adb_devices(output: &str) -> Vec<AdbEmulator> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let serial = parts.next()?;
            let state = parts.next()?;
            if !serial.starts_with("emulator-") {
                return None;
            }
            let state = match state {
                "device" => DeviceState::Booted,
                "offline" => DeviceState::Booting,
                _ => DeviceState::Unknown,
            };
            Some(AdbEmulator {
                serial: serial.to_string(),
                state,
            })
        })
        .collect()
}

/// Parses `adb -s <serial> emu avd name`: the AVD name is the first line.
pub fn parse_emu_avd_name(output: &str) -> Option<String> {
    let first = output.lines().next()?.trim();
    if first.is_empty() || first.starts_with("KO") || first.contains(' ') {
        None
    } else {
        Some(first.to_string())
    }
}

/// Parses the relevant keys of an AVD `config.ini`.
pub fn parse_avd_config(ini: &str) -> AvdConfig {
    let mut cfg = AvdConfig::default();
    for line in ini.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "avd.ini.displayname" if !value.is_empty() => cfg.display_name = Some(value.into()),
            "target" => {
                if let Some(api) = value.strip_prefix("android-") {
                    cfg.os_version = Some(format!("Android API {api}"));
                }
            }
            _ => {}
        }
    }
    cfg
}

/// Merges known AVDs with the running instances (`avd name -> emulator`).
pub fn merge_devices(
    avds: &[String],
    running: &HashMap<String, AdbEmulator>,
    configs: &HashMap<String, AvdConfig>,
) -> Vec<Device> {
    avds.iter()
        .map(|avd| {
            let config = configs.get(avd);
            let instance = running.get(avd);
            Device {
                id: avd.clone(),
                name: config
                    .and_then(|c| c.display_name.clone())
                    .unwrap_or_else(|| avd.replace('_', " ")),
                platform: Platform::Android,
                state: instance.map_or(DeviceState::Shutdown, |e| e.state),
                os_version: config.and_then(|c| c.os_version.clone()),
                serial: instance.map(|e| e.serial.clone()),
            }
        })
        .collect()
}

pub struct AndroidProvider {
    tools: ToolPaths,
}

impl AndroidProvider {
    pub fn new(tools: ToolPaths) -> Self {
        Self { tools }
    }

    async fn adb(&self, args: &[&str]) -> Result<String, String> {
        process::run(&self.tools.adb, args).await
    }

    /// Maps AVD names to their running adb instance.
    async fn running_by_avd(&self) -> Result<HashMap<String, AdbEmulator>, String> {
        let output = self.adb(&["devices"]).await?;
        let mut map = HashMap::new();
        for emu in parse_adb_devices(&output) {
            // `emu avd name` fails while the console is not up yet; skip those.
            if let Ok(out) = self.adb(&["-s", &emu.serial, "emu", "avd", "name"]).await
                && let Some(name) = parse_emu_avd_name(&out)
            {
                map.insert(name, emu);
            }
        }
        Ok(map)
    }

    fn read_config(&self, avd: &str) -> AvdConfig {
        let path = self
            .tools
            .avd_home
            .join(format!("{avd}.avd"))
            .join("config.ini");
        std::fs::read_to_string(path)
            .map(|s| parse_avd_config(&s))
            .unwrap_or_default()
    }
}

impl EmulatorProvider for AndroidProvider {
    async fn list(&self) -> Result<Vec<Device>, String> {
        let avds = parse_avd_list(&process::run(&self.tools.emulator, &["-list-avds"]).await?);
        // adb may be missing or its server may fail; still list AVDs as shut down.
        let running = self.running_by_avd().await.unwrap_or_default();
        let configs = avds
            .iter()
            .map(|a| (a.clone(), self.read_config(a)))
            .collect();
        Ok(merge_devices(&avds, &running, &configs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_avd_list_ignoring_log_lines() {
        let avds = parse_avd_list(include_str!("fixtures/emulator_list_avds.txt"));
        assert_eq!(avds, vec!["Pixel_10_Pro_XL", "Pixel_10a"]);
    }

    #[test]
    fn parses_adb_devices_emulators_only() {
        let devices = parse_adb_devices(include_str!("fixtures/adb_devices.txt"));
        assert_eq!(
            devices,
            vec![
                AdbEmulator {
                    serial: "emulator-5554".into(),
                    state: DeviceState::Booted
                },
                AdbEmulator {
                    serial: "emulator-5556".into(),
                    state: DeviceState::Booting
                },
            ]
        );
    }

    #[test]
    fn parses_adb_devices_empty_list() {
        assert!(parse_adb_devices("List of devices attached\n\n").is_empty());
    }

    #[test]
    fn parses_emu_avd_name_with_crlf() {
        assert_eq!(
            parse_emu_avd_name("Pixel_10a\r\nOK\r\n").as_deref(),
            Some("Pixel_10a")
        );
        assert_eq!(parse_emu_avd_name("\r\n"), None);
        assert_eq!(parse_emu_avd_name("KO: unknown command\r\n"), None);
    }

    #[test]
    fn parses_avd_config() {
        let cfg = parse_avd_config(include_str!("fixtures/avd_config.ini"));
        assert_eq!(cfg.display_name.as_deref(), Some("Pixel 10a"));
        assert_eq!(cfg.os_version.as_deref(), Some("Android API 37.2"));
    }

    #[test]
    fn merges_running_state_and_config() {
        let avds = vec!["Pixel_10_Pro_XL".to_string(), "Pixel_10a".to_string()];
        let running = HashMap::from([(
            "Pixel_10a".to_string(),
            AdbEmulator {
                serial: "emulator-5554".into(),
                state: DeviceState::Booted,
            },
        )]);
        let configs = HashMap::from([(
            "Pixel_10a".to_string(),
            AvdConfig {
                display_name: Some("Pixel 10a".into()),
                os_version: Some("Android API 37.2".into()),
            },
        )]);

        let devices = merge_devices(&avds, &running, &configs);

        assert_eq!(
            devices,
            vec![
                Device {
                    id: "Pixel_10_Pro_XL".into(),
                    name: "Pixel 10 Pro XL".into(),
                    platform: Platform::Android,
                    state: DeviceState::Shutdown,
                    os_version: None,
                    serial: None,
                },
                Device {
                    id: "Pixel_10a".into(),
                    name: "Pixel 10a".into(),
                    platform: Platform::Android,
                    state: DeviceState::Booted,
                    os_version: Some("Android API 37.2".into()),
                    serial: Some("emulator-5554".into()),
                },
            ]
        );
    }
}
