// FAMREV AI desktop shell: everything lives in lib.rs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    famrev_desktop_lib::run()
}
