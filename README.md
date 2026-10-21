# Emulator Bar

A macOS menu-bar app (Tauri v2 + React + TypeScript) to list and control local
Android emulators (AVDs) and iOS simulators from a single popover.

## Requirements

- macOS with Xcode command line tools (`/usr/bin/xcrun simctl`).
- Android SDK with `platform-tools` and `emulator`. The SDK is resolved from
  `ANDROID_HOME`, then `ANDROID_SDK_ROOT`, then `~/Library/Android/sdk`
  (GUI apps do not inherit your shell `PATH`).
- Rust (stable), Node.js and pnpm.

## Run

```sh
pnpm install
pnpm tauri dev          # development
pnpm tauri build        # release bundle (.app / .dmg)
```

The app has no Dock icon. Left-click the tray icon to toggle the popover;
right-click it for the menu (Launch at login, Quit).

### Launch at login

"Launch at login" in the tray menu registers a LaunchAgent
(`~/Library/LaunchAgents`) that points to the running executable. It is off by
default. Do not enable it from `pnpm tauri dev`: the agent would point to the
debug binary. Build the app (`pnpm tauri build`), install `Emulator Bar.app` to
`/Applications`, launch it from there, and enable it from the tray menu.

## Test

```sh
cd src-tauri
cargo test                       # parser unit tests
cargo test -- --ignored --nocapture   # lists real devices on this machine
```
