# Emulator Bar

A macOS menu-bar app to list and control your local Android emulators (AVDs)
and iOS simulators from a single popover: boot, shut down, open a URL or deep
link, install an `.apk` / `.app`, take a screenshot, and wipe devices, plus
bulk actions across every booted device.

Built with Tauri v2 + React + TypeScript.

## Install

```sh
brew install --cask ale24dev/tap/emulator-bar
```

Apple Silicon (arm64) only.

### First launch (unsigned app)

The app is not signed or notarized, so Gatekeeper blocks the first launch.
Either right-click `Emulator Bar.app` in `/Applications` and choose **Open**,
or remove the quarantine attribute:

```sh
xattr -dr com.apple.quarantine "/Applications/Emulator Bar.app"
```

## Requirements

- macOS with Xcode command line tools (`/usr/bin/xcrun simctl`) for iOS
  simulators.
- Android SDK with `platform-tools` and `emulator` for Android emulators. The
  SDK is resolved from `ANDROID_HOME`, then `ANDROID_SDK_ROOT`, then
  `~/Library/Android/sdk` (GUI apps do not inherit your shell `PATH`).

## Usage

The app has no Dock icon. Left-click the tray icon to toggle the popover;
right-click it for the menu (Launch at login, Quit).

### Launch at login

"Launch at login" in the tray menu registers a LaunchAgent
(`~/Library/LaunchAgents`) that points to the running executable. It is off by
default. Enable it from the installed app in `/Applications`, not from
`pnpm tauri dev` (the agent would point to the debug binary).

## Build from source

Requires Rust (stable), Node.js and pnpm.

```sh
pnpm install
pnpm tauri dev          # development
pnpm tauri build        # release bundle: src-tauri/target/release/bundle/macos/Emulator Bar.app
```

## Test

```sh
cd src-tauri
cargo test                            # parser unit tests
cargo test -- --ignored --nocapture   # lists real devices on this machine
```

## License

[MIT](LICENSE)
