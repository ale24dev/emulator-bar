//! Pure helpers shared by device actions (validation, quoting, file names).

use std::path::{Path, PathBuf};

/// Accepts URLs and deep links with a scheme, e.g. `https://x.dev` or `myapp://home`.
pub fn validate_url(url: &str) -> Result<&str, String> {
    let url = url.trim();
    let valid_scheme = url.split_once(':').is_some_and(|(scheme, rest)| {
        !rest.is_empty()
            && scheme
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    });
    if valid_scheme {
        Ok(url)
    } else {
        Err("Enter a URL with a scheme, e.g. https://example.com or myapp://path".into())
    }
}

/// Quotes a value for the remote `sh` that `adb shell` runs.
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// Ensures `path` exists and has the expected extension (`apk`, `app`).
pub fn validate_package(path: &str, extension: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    let matches = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(extension));
    if !matches {
        return Err(format!(
            "Expected a .{extension} file, got {}",
            path.display()
        ));
    }
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    Ok(path)
}

/// `<dir>/<device-name>-<timestamp>.png` with a filesystem-safe device name.
pub fn screenshot_path(dir: &Path, device_name: &str, timestamp: &str) -> PathBuf {
    let safe: Vec<&str> = device_name
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
        .filter(|p| !p.is_empty())
        .collect();
    let name = if safe.is_empty() {
        "device".to_string()
    } else {
        safe.join("-")
    };
    dir.join(format!("{name}-{timestamp}.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_urls_and_deep_links() {
        assert_eq!(
            validate_url("  https://example.com/a?b=1&c=2 ").unwrap(),
            "https://example.com/a?b=1&c=2"
        );
        assert_eq!(
            validate_url("myapp://profile/42").unwrap(),
            "myapp://profile/42"
        );
        assert!(validate_url("").is_err());
        assert!(validate_url("example.com").is_err());
        assert!(validate_url("://nope").is_err());
        assert!(validate_url("has space://x").is_err());
    }

    #[test]
    fn quotes_for_remote_shell() {
        assert_eq!(
            shell_quote("https://a.dev/?x=1&y=2"),
            "'https://a.dev/?x=1&y=2'"
        );
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn validates_package_extension_and_existence() {
        let dir = std::env::temp_dir().join(format!("emubar-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("My.app")).unwrap();
        std::fs::write(dir.join("app.apk"), b"x").unwrap();

        let apk = dir.join("app.apk");
        assert_eq!(validate_package(apk.to_str().unwrap(), "apk").unwrap(), apk);
        assert!(validate_package(dir.join("My.app").to_str().unwrap(), "app").is_ok());
        let err = validate_package(apk.to_str().unwrap(), "app").unwrap_err();
        assert!(err.contains(".app"), "{err}");
        assert!(validate_package(dir.join("missing.apk").to_str().unwrap(), "apk").is_err());

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn builds_safe_screenshot_paths() {
        let path = screenshot_path(
            Path::new("/Users/me/Desktop"),
            "iPhone 18 Pro / Max",
            "20261021-120000",
        );
        assert_eq!(
            path,
            PathBuf::from("/Users/me/Desktop/iPhone-18-Pro-Max-20261021-120000.png")
        );
    }
}
