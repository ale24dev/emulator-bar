//! Absolute tool path resolution. GUI apps on macOS do not inherit the shell
//! `PATH`, so every binary is resolved explicitly.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPaths {
    pub adb: PathBuf,
    pub emulator: PathBuf,
    pub xcrun: PathBuf,
    /// Directory holding `<name>.ini` / `<name>.avd` AVD definitions.
    pub avd_home: PathBuf,
}

impl ToolPaths {
    /// Resolves paths from the real process environment.
    pub fn from_env() -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        Self::resolve(|key| std::env::var(key).ok(), home.as_deref())
    }

    /// Pure resolution logic: `env` looks up environment variables.
    pub fn resolve(env: impl Fn(&str) -> Option<String>, home: Option<&Path>) -> Self {
        let non_empty = |key: &str| env(key).filter(|v| !v.trim().is_empty()).map(PathBuf::from);
        let home = home
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("/"));

        let sdk = non_empty("ANDROID_HOME")
            .or_else(|| non_empty("ANDROID_SDK_ROOT"))
            .unwrap_or_else(|| home.join("Library/Android/sdk"));

        let avd_home = non_empty("ANDROID_AVD_HOME")
            .or_else(|| non_empty("ANDROID_USER_HOME").map(|p| p.join("avd")))
            .unwrap_or_else(|| home.join(".android/avd"));

        Self {
            adb: sdk.join("platform-tools/adb"),
            emulator: sdk.join("emulator/emulator"),
            xcrun: PathBuf::from("/usr/bin/xcrun"),
            avd_home,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn falls_back_to_default_sdk_location() {
        let paths = ToolPaths::resolve(env_of(&[]), Some(Path::new("/Users/me")));
        assert_eq!(
            paths.adb,
            PathBuf::from("/Users/me/Library/Android/sdk/platform-tools/adb")
        );
        assert_eq!(
            paths.emulator,
            PathBuf::from("/Users/me/Library/Android/sdk/emulator/emulator")
        );
        assert_eq!(paths.xcrun, PathBuf::from("/usr/bin/xcrun"));
        assert_eq!(paths.avd_home, PathBuf::from("/Users/me/.android/avd"));
    }

    #[test]
    fn android_home_wins_over_sdk_root() {
        let env = env_of(&[("ANDROID_HOME", "/opt/sdk"), ("ANDROID_SDK_ROOT", "/other")]);
        let paths = ToolPaths::resolve(env, Some(Path::new("/Users/me")));
        assert_eq!(paths.adb, PathBuf::from("/opt/sdk/platform-tools/adb"));
    }

    #[test]
    fn uses_sdk_root_and_ignores_blank_android_home() {
        let env = env_of(&[("ANDROID_HOME", "  "), ("ANDROID_SDK_ROOT", "/other")]);
        let paths = ToolPaths::resolve(env, Some(Path::new("/Users/me")));
        assert_eq!(paths.emulator, PathBuf::from("/other/emulator/emulator"));
    }

    #[test]
    fn avd_home_honours_android_avd_home_then_user_home() {
        let paths = ToolPaths::resolve(env_of(&[("ANDROID_AVD_HOME", "/avds")]), None);
        assert_eq!(paths.avd_home, PathBuf::from("/avds"));
        let paths = ToolPaths::resolve(env_of(&[("ANDROID_USER_HOME", "/u")]), None);
        assert_eq!(paths.avd_home, PathBuf::from("/u/avd"));
    }
}
