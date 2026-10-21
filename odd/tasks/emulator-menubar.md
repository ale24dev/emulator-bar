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
- [x] T1 Scaffold Tauri v2 + React/TS app, git init, feature branch `feat/emulator-menubar`, menu-bar-only shell (tray, popover, no Dock). Route: delegated.
- [x] T2 Rust core: tool path resolution, `EmulatorProvider` trait, iOS provider (parse `simctl list devices --json`) + Android provider (list-avds, adb devices, avd name mapping), with unit tests for parsers. Route: delegated.
- [x] T3 Rust actions: boot, shutdown, open URL, install, screenshot, erase, bulk actions, exposed as Tauri commands. Route: delegated.
- [x] T4 Frontend: device list grouped by platform, status badges, per-device actions, deep link input, file picker for install, bulk actions, refresh/polling. Route: delegated.
- [x] T5 Verification: cargo test, cargo clippy, pnpm build, tauri build (debug) smoke. Route: delegated.
- [x] T6 Launch at login: `tauri-plugin-autostart` (LaunchAgent) with a "Launch at login" check item in the tray menu above Quit; default off; README note. Route: delegated (writer).
- [x] T7 App + tray icons: replace default Tauri icons with the "Split" design (phone halved, Android green / Apple graphite); SVG sources in `src-tauri/icons/source/`; tray stays a template image. Route: delegated (writer).
- [x] T8 Release v0.1.0 + Homebrew distribution: `bundle.targets = ["app"]` (DMG fails on this hdiutil), MIT LICENSE, README install/Gatekeeper notes; publish `ale24dev/emulator-bar` (release zip) and `ale24dev/homebrew-tap` (`Casks/emulator-bar.rb`); verify `brew install --cask`. Route: delegated (writer, remote authorized for ale24dev only).

## Acceptance criteria
- Tray icon appears; clicking toggles popover; no Dock icon.
- All 8 iOS simulators and 2 AVDs (Pixel_10_Pro_XL, Pixel_10a) are listed with correct state.
- Each action works on a booted device; errors surface in UI.

## Checks
- `cargo test` (parsers), `cargo clippy`, `pnpm build`, `pnpm tauri build --debug`.

## Review
RDD: off (global). Ordinary checks only.

## Progress / Evidence
Branch `feat/emulator-menubar` (base commit `d8d11f6` on `main`: feature document).

| Task | Commit | Route | Evidence |
|------|--------|-------|----------|
| T1 | `9643828` feat: scaffold tauri menu-bar shell with tray popover | delegated (writer) | `cargo build`, `cargo clippy -D warnings`, `pnpm build` green. Test-first exception: scaffold/config, no runnable behavior test. |
| T2 | `998b59f` feat: add device providers with simctl and adb parsers | delegated (writer) | RED: 11 parser tests failed on `todo!()` stubs; GREEN: 15/15 passed. Real-device list (ignored test): 8 iOS 27.0 sims + Pixel_10_Pro_XL (shutdown) + Pixel_10a (booted, emulator-5554). |
| T3 | `e4942b7` feat: add device actions and bulk tauri commands | delegated (writer) | RED: 4 helper tests failed; GREEN: 19/19. Ignored smoke: screenshot of booted Pixel 10a (364 KB PNG, temp dir). Boot/shutdown/erase/install/openurl not exercised against real devices (destructive or state-changing). |
| T4 | `3764fc9` feat: add popover device list ui with actions and bulk controls | delegated (writer) | `tsc && vite build` green. Test-first exception: no JS test runner in scope; UI checked by type-check/build. |
| T5 | (this commit) | delegated (writer) | `cargo test` 19 passed / 2 ignored; `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean; `pnpm build` green; `pnpm tauri build --debug` produced `Emulator Bar.app` + `.dmg`; bundled app ran 5 s without errors and System Events reports `background only = true` (no Dock icon). |
| T6 | `867168d` feat: add launch at login toggle to tray menu | delegated (writer) | `cargo test` 19 passed / 2 ignored; `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean. Test-first exception: tray menu wiring + OS LaunchAgent registration has no runnable deterministic test; toggle deliberately not exercised on this machine (would register a LaunchAgent for the debug binary). |
| T7 | (this commit) feat: replace default icons with split phone design | delegated (writer) | SVGs rasterized with `@resvg/resvg-js` (scratch install, no repo dep); `pnpm tauri icon` regenerated icns/ico/PNGs (generated android/ios folders removed; new `64x64.png` kept); tray.png 44x44 black-on-transparent, `icon_as_template(true)` already set; 1024 and tray renders inspected visually, corner alpha 0; `cargo build` + `cargo clippy -- -D warnings` clean after forced rebuild. Test-first exception: asset swap, no runnable behavior test. |
| T8 | `55204ac` chore: prepare v0.1.0 release and homebrew distribution | delegated (writer, remote: ale24dev only) | `pnpm tauri build` produced `Emulator Bar.app` (4.64 MiB, arm64, minos 11.0, LSMinimumSystemVersion 10.13). Zip sha256 `807033b2...e705b` matches the downloaded release asset. Repo https://github.com/ale24dev/emulator-bar (main fast-forwarded, tag `v0.1.0`), release https://github.com/ale24dev/emulator-bar/releases/tag/v0.1.0, tap https://github.com/ale24dev/homebrew-tap (`5fd3500` feat: add emulator-bar cask). `brew style --cask`: no offenses (after autocorrect to `depends_on :macos`, Big Sur floor is below Homebrew's minimum). `brew audit --cask`: clean; `--new --online` fails only on notability and unsigned signature (homebrew/cask-only policies). `brew install --cask ale24dev/tap/emulator-bar` moved the app to /Applications; after `xattr -dr com.apple.quarantine` the process runs. Test-first exception: packaging/distribution, no runnable behavior test. |

Not verified: tray click/popover positioning and hide-on-blur by hand (needs interactive GUI); per-action behavior on real devices beyond list + screenshot.

### Decisions
- `Device` adds `serial` (Android adb serial) on top of the planned fields; actions re-resolve the serial at call time.
- Android `os_version` comes from `<avd_home>/<name>.avd/config.ini` (`target=android-X` -> "Android API X"); display name from `avd.ini.displayname`.
- Android boot is fire-and-forget (`emulator -avd` detached); the UI keeps an optimistic "Booting" state (90 s TTL) until adb reports it.
- Android wipe uses `emulator -avd <name> -wipe-data`, which cold boots the emulator.
- `set_hide_on_blur` command keeps the popover open while the native file dialog has focus.
- `am start` failures are detected from stdout (`Error...`) because adb exits 0.
- Launch at login is driven from Rust only (`ManagerExt::autolaunch()`), so no `autostart:*` capability permission is needed; after each toggle the check mark is resynced from `is_enabled()`.

## Next step
v0.1.0 is published via `brew install --cask ale24dev/tap/emulator-bar`. Future releases: bump version in tauri.conf.json/Cargo.toml/package.json, rebuild, zip, new release, update cask `version` + `sha256`.

Previous: Manual GUI check with `pnpm tauri dev` (tray toggle, actions on a booted device). Delivery (push/PR) is the user's decision; the branch exceeds ~400 lines, so the chain strategy question (ask-on-risk) is still pending before any PR.
