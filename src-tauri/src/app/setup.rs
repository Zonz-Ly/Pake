use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{tray::TrayIconBuilder, AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub fn set_system_tray(app: &AppHandle, show_system_tray: bool) -> tauri::Result<()> {
    if !show_system_tray {
        app.remove_tray_by_id("pake-tray");
        return Ok(());
    }
    app.app_handle().remove_tray_by_id("pake-tray");

    let tray = TrayIconBuilder::new()
        // 不再设置 .menu()
        .on_tray_icon_event(|app, event| {
            if event.click_type == tauri::tray::ClickType::Left {
                if let Some(window) = app.get_webview_window("pake") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
        })
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;

    tray.set_icon_as_template(false)?;
    Ok(())
}

pub fn set_global_shortcut(app: &AppHandle, shortcut: String) -> tauri::Result<()> {
    if shortcut.is_empty() {
        return Ok(());
    }

    let app_handle = app.clone();
    let shortcut_hotkey = Shortcut::from_str(&shortcut).unwrap();
    let last_triggered = Arc::new(Mutex::new(Instant::now()));

    app_handle
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler({
                    let last_triggered = Arc::clone(&last_triggered);
                    move |app, event, _shortcut| {
                        let mut last_triggered = last_triggered.lock().unwrap();
                        if Instant::now().duration_since(*last_triggered)
                            < Duration::from_millis(300)
                        {
                            return;
                        }
                        *last_triggered = Instant::now();

                        if shortcut_hotkey.eq(event) {
                            if let Some(window) = app.get_webview_window("pake") {
                                let is_visible = window.is_visible().unwrap();
                                if is_visible {
                                    window.hide().unwrap();
                                } else {
                                    window.show().unwrap();
                                    window.set_focus().unwrap();
                                }
                            }
                        }
                    }
                })
                .build(),
        )
        .expect("Failed to set global shortcut");

    app.global_shortcut().register(shortcut_hotkey).unwrap();

    Ok(())
}
