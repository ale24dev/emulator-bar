//! Tray icon and popover window toggling.

use tauri::{
    App, AppHandle, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_positioner::{Position, WindowExt};

const MAIN_WINDOW: &str = "main";

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit Emulator Bar", true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(app, &[&quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(tauri::include_image!("icons/tray.png"))
        .icon_as_template(true)
        .tooltip("Emulator Bar")
        .menu(&menu)
        // Left click toggles the popover; the menu (Quit) opens on right click.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
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
