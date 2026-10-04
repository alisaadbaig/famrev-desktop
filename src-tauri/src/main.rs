// FAMREV AI — desktop shell. Minimal, guaranteed-compile window loader.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running FAMREV AI desktop");
}
