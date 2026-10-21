//! Emulator Bar: a macOS menu-bar controller for Android emulators and iOS simulators.

mod commands;
mod devices;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .manage(commands::Providers::new(
            devices::tools::ToolPaths::from_env(),
        ))
        .invoke_handler(tauri::generate_handler![
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
            if let tauri::WindowEvent::Focused(false) = event {
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
