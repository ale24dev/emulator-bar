//! Emulator Bar: a macOS menu-bar controller for Android emulators and iOS simulators.

mod commands;
mod devices;
mod tray;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

/// When false, the popover stays open on blur (e.g. while a file dialog is up).
struct HideOnBlur(AtomicBool);

#[tauri::command]
fn set_hide_on_blur(state: tauri::State<'_, HideOnBlur>, enabled: bool) {
    state.0.store(enabled, Ordering::Relaxed);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_positioner::init())
        .manage(HideOnBlur(AtomicBool::new(true)))
        .manage(commands::Providers::new(
            devices::tools::ToolPaths::from_env(),
        ))
        .invoke_handler(tauri::generate_handler![
            set_hide_on_blur,
            commands::list_devices,
            commands::boot_device,
            commands::shutdown_device,
            commands::open_url,
            commands::install_app,
            commands::take_screenshot,
            commands::erase_device,
            commands::shutdown_all,
            commands::open_url_all_booted,
        ])
        .setup(|app| {
            // Menu-bar only: no Dock icon, no app switcher entry.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            tray::setup(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Behave like a popover: hide when focus moves elsewhere.
            if let tauri::WindowEvent::Focused(false) = event
                && window.state::<HideOnBlur>().0.load(Ordering::Relaxed)
            {
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
