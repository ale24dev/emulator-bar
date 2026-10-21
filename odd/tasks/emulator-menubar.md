# Feature: emulator-menubar

## Objective
macOS menu-bar app (Tauri v2) to list and control all local Android emulators (AVDs) and iOS simulators from one popover.

## Problem / Why
Managing emulators means juggling Android Studio, Simulator.app and CLI commands. A menu-bar controller gives one-click access to every device.

## Scope
- Tauri v2 + React + TypeScript (Vite), pnpm.
- Menu-bar only: tray icon, no Dock icon (ActivationPolicy::Accessory), popover window toggled by tray click, hidden on blur.
- The app is a remote control: emulators keep opening in their own windows (no embedding).
- Rust `EmulatorProvider` trait with `AndroidProvider` (adb + emulator) and `IosProvider` (xcrun simctl).
- Actions: list with state, boot, shutdown, open URL/deep link, install .apk/.app, screenshot to ~/Desktop, wipe/erase.
- Bulk actions: shutdown all, open deep link on all booted devices.

## Constraints
- GUI apps do not inherit shell PATH: resolve tool paths explicitly (ANDROID_HOME / ANDROID_SDK_ROOT, fallback ~/Library/Android/sdk; xcrun at /usr/bin/xcrun).
- Android booted state: map `adb devices` emulator-XXXX serials to AVD names via `adb -s <serial> emu avd name`.
- Command execution must never block the UI thread (async commands).
- Parsing logic separated from process execution so it is unit-testable.
- Artifacts in English.

## Delivery strategy
ask-on-risk. Forecast: ~1200 authored lines (exceeds 400 → chain strategy question deferred; no PR planned yet, local commits only).

## Tasks
- [ ] T1 Scaffold Tauri v2 + React/TS app, git init, feature branch `feat/emulator-menubar`, menu-bar-only shell (tray, popover, no Dock). Route: delegated.
- [ ] T2 Rust core: tool path resolution, `EmulatorProvider` trait, iOS provider (parse `simctl list devices --json`) + Android provider (list-avds, adb devices, avd name mapping), with unit tests for parsers. Route: delegated.
- [ ] T3 Rust actions: boot, shutdown, open URL, install, screenshot, erase, bulk actions, exposed as Tauri commands. Route: delegated.
- [ ] T4 Frontend: device list grouped by platform, status badges, per-device actions, deep link input, file picker for install, bulk actions, refresh/polling. Route: delegated.
- [ ] T5 Verification: cargo test, cargo clippy, pnpm build, tauri build (debug) smoke. Route: delegated.

## Acceptance criteria
- Tray icon appears; clicking toggles popover; no Dock icon.
- All 8 iOS simulators and 2 AVDs (Pixel_10_Pro_XL, Pixel_10a) are listed with correct state.
- Each action works on a booted device; errors surface in UI.

## Checks
- `cargo test` (parsers), `cargo clippy`, `pnpm build`, `pnpm tauri build --debug`.

## Review
RDD: off (global). Ordinary checks only.

## Progress / Evidence
(pending)

## Next step
T1.
