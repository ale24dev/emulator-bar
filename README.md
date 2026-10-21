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
right-click it for the Quit menu.

## Test

```sh
cd src-tauri
cargo test                       # parser unit tests
cargo test -- --ignored --nocapture   # lists real devices on this machine
```
