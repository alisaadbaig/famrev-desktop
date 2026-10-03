// FAMREV AI — desktop shell (Tauri). Loads the live web UI in a clean native window.
// Minimal, dependency-light version to guarantee a clean first build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running FAMREV AI desktop");
}
