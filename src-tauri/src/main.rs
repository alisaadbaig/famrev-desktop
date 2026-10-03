// FAMREV AI — desktop shell (Tauri). Loads the live web UI, adds native
// global hotkey, system tray, and keeps a single resident window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem,
};

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_window("main") {
        if w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
            let _ = w.set_focus();
            let _ = w.unminimize();
        }
    }
}

fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Open FAMREV AI"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));
    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => toggle_window(app),
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => toggle_window(app),
                "quit" => std::process::exit(0),
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|e| {
            // Closing the window hides to tray instead of quitting (resident app).
            if let tauri::WindowEvent::CloseRequested { api, .. } = e.event() {
                let _ = e.window().hide();
                api.prevent_close();
            }
        })
        .setup(|app| {
            let handle = app.handle();
            // Global hotkey: Ctrl+Shift+Space summons/hides the app.
            let mut gs = app.global_shortcut_manager();
            let h2 = handle.clone();
            let _ = gs.register("CmdOrControl+Shift+Space", move || toggle_window(&h2));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running FAMREV AI desktop");
}
