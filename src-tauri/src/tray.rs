//! Tray icon and popover window toggling.

use tauri::{
    App, AppHandle, Manager,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_positioner::{Position, WindowExt};

const MAIN_WINDOW: &str = "main";
const AUTOSTART_ID: &str = "autostart";

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or_else(|err| {
        eprintln!("failed to read launch-at-login state: {err}");
        false
    });
    let autostart = CheckMenuItem::with_id(
        app,
        AUTOSTART_ID,
        "Launch at login",
        true,
        autostart_enabled,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Emulator Bar", true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(app, &[&autostart, &separator, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(tauri::include_image!("icons/tray.png"))
        .icon_as_template(true)
        .tooltip("Emulator Bar")
        .menu(&menu)
        // Left click toggles the popover; the menu (Quit) opens on right click.
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            AUTOSTART_ID => toggle_autostart(app, &autostart),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_popover(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Flips launch-at-login and syncs the check mark with the real OS state.
fn toggle_autostart(app: &AppHandle, item: &CheckMenuItem<tauri::Wry>) {
    let manager = app.autolaunch();
    let currently_enabled = manager.is_enabled().unwrap_or(false);
    let result = if currently_enabled {
        manager.disable()
    } else {
        manager.enable()
    };
    if let Err(err) = result {
        eprintln!("failed to update launch-at-login: {err}");
    }
    let actual = manager.is_enabled().unwrap_or_else(|err| {
        eprintln!("failed to read launch-at-login state: {err}");
        currently_enabled
    });
    let _ = item.set_checked(actual);
}

fn toggle_popover(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.move_window(Position::TrayBottomCenter);
        let _ = window.show();
        let _ = window.set_focus();
    }
}
