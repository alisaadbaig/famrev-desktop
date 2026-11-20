fn main() {
    // The app's own commands. Listing them here generates the permissions `allow-<command>` that
    // capabilities/remote.json grants to the FAMREV pages (and to nothing else).
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "qb_show", "qb_hide", "qb_toggle", "qb_resize", "open_main", "open_program", "open_link", "capture_screen", "app_info",
        ]),
    ))
    .expect("failed to run tauri-build");
}
