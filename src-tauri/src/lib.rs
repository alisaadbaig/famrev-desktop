//! FAMREV AI desktop app (Tauri 2).
//!
//! Windows:
//!   main      FAMREV in its own window. Closing it hides it to the tray (Quit is in the tray menu).
//!   quickbox  the quick box: small, frameless, ALWAYS ON TOP, not in the taskbar. It stays above every program, also
//!             when the main window is minimized or hidden. It loads <server>/?view=quick (the web app shows only the
//!             quick box there) and shares the main window's sign-in.
//! Global hotkey Ctrl+Alt+Space (from any program) shows / hides the quick box on the screen the mouse is on.
//! Tray: Open FAMREV · Quick box · Quit. A second start of the app brings the running one forward.
//!
//! What the FAMREV pages may ask (capabilities/remote.json; only https://chat.famrev.ai and https://uat.famrev.ai):
//!   qb_show / qb_hide / qb_toggle / qb_resize, open_main(route, conv), open_program(name from a FIXED list),
//!   capture_screen (one picture of the screen the quick box is on, the box hidden while it is taken), app_info.
//! Nothing else: no files, no shell, no other windows. (2.0.2: web links open in the default browser, open_link.)

use base64::Engine;
use serde::Serialize;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// The FAMREV server this build opens. Set FAMREV_URL when building (the UAT build uses https://uat.famrev.ai).
pub const SERVER: &str = match option_env!("FAMREV_URL") {
    Some(u) => u,
    None => "https://chat.famrev.ai",
};
/// WebView2 arguments for EVERY window. Both windows share one data folder (one sign-in), and WebView2 will not
/// create a second window there with different arguments: the quick box then fails, setup fails, and the app quits
/// at start (2.0.2 fix). Keep main and quickbox on this same value.
const BROWSER_ARGS: &str =
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --enable-features=msSingleSignOnOSForPrimaryAccountIsShared";
const QB_W: f64 = 660.0;
const QB_H_MIN: f64 = 96.0;
const QB_H_MAX: f64 = 760.0;
/// The routes open_main accepts (the web page maps them to its own pages).
const ROUTES: [&str; 6] = ["chat", "email", "phone", "fax", "notes", "files"];
/// Programs open_program may start: a fixed list, never a path or arguments from the page.
const PROGRAMS: [(&str, &str); 5] = [("outlook", "outlook"), ("excel", "excel"), ("word", "winword"), ("teams", "msteams:"), ("calculator", "calc")];

#[derive(Default)]
struct QbState {
    height: Mutex<f64>,
}

#[derive(Serialize, Clone)]
struct OpenEvent {
    route: String,
    conv: Option<String>,
}

#[derive(Serialize)]
struct AppInfo {
    version: String,
    server: String,
    os: String,
}

fn hotkey() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space)
}

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}

fn show_main(app: &AppHandle) {
    if let Some(w) = main_window(app) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn quickbox(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(w) = app.get_webview_window("quickbox") {
        return Ok(w);
    }
    let url = format!("{}/?view=quick", SERVER.trim_end_matches('/'));
    WebviewWindowBuilder::new(app, "quickbox", WebviewUrl::External(url.parse().expect("server address")))
        .title("FAMREV quick box")
        .inner_size(QB_W, 132.0)
        .min_inner_size(420.0, QB_H_MIN)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .visible(false)
        .disable_drag_drop_handler() // files dropped on the box reach the page (its own drop handler)
        .additional_browser_args(BROWSER_ARGS) // must match the main window, or the box can't be created
        .build()
}

/// Bottom centre of the screen the mouse is on (else the main window's, else the primary).
fn place_quickbox(app: &AppHandle, w: &WebviewWindow) {
    let mon = app
        .cursor_position()
        .ok()
        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
        .or_else(|| w.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(m) = mon {
        let scale = m.scale_factor();
        let size = w.outer_size().unwrap_or(PhysicalSize::new((QB_W * scale) as u32, (132.0 * scale) as u32));
        let (mx, my) = (m.position().x, m.position().y);
        let (mw, mh) = (m.size().width as i32, m.size().height as i32);
        let x = mx + (mw - size.width as i32) / 2;
        let y = my + mh - size.height as i32 - (72.0 * scale) as i32; // above the taskbar
        let _ = w.set_position(PhysicalPosition::new(x, y));
    }
}

fn qb_show_impl(app: &AppHandle) -> tauri::Result<()> {
    let w = quickbox(app)?;
    if !w.is_visible().unwrap_or(false) {
        place_quickbox(app, &w);
    }
    w.show()?;
    w.set_always_on_top(true)?;
    w.set_focus()?;
    let _ = app.emit_to("quickbox", "famrev-qb-shown", ());
    Ok(())
}

fn qb_toggle_impl(app: &AppHandle) -> tauri::Result<()> {
    let w = quickbox(app)?;
    if w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false) {
        w.hide()
    } else {
        qb_show_impl(app)
    }
}

// ------------------------------------------------------------------------------------------------- commands
#[tauri::command]
fn qb_show(app: AppHandle) -> Result<(), String> {
    qb_show_impl(&app).map_err(|e| e.to_string())
}

#[tauri::command]
fn qb_hide(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("quickbox") {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn qb_toggle(app: AppHandle) -> Result<(), String> {
    qb_toggle_impl(&app).map_err(|e| e.to_string())
}

/// The page asks for the height it needs (the answer grows); the bottom edge stays where it is.
#[tauri::command]
fn qb_resize(app: AppHandle, state: tauri::State<QbState>, height: f64) -> Result<(), String> {
    let w = app.get_webview_window("quickbox").ok_or("no quick box")?;
    let h = height.clamp(QB_H_MIN, QB_H_MAX);
    let scale = w.scale_factor().unwrap_or(1.0);
    let old = w.outer_size().map_err(|e| e.to_string())?;
    let pos = w.outer_position().map_err(|e| e.to_string())?;
    let width = old.width as f64 / scale;
    w.set_size(tauri::LogicalSize::new(width, h)).map_err(|e| e.to_string())?;
    let new_h = (h * scale) as i32;
    let _ = w.set_position(PhysicalPosition::new(pos.x, pos.y + old.height as i32 - new_h));
    *state.height.lock().unwrap() = h;
    Ok(())
}

/// Bring the main window forward on one of FAMREV's pages (and optionally a chat); the page does the opening.
#[tauri::command]
fn open_main(app: AppHandle, route: String, conv: Option<String>) -> Result<(), String> {
    if !ROUTES.contains(&route.as_str()) {
        return Err("unknown page".into());
    }
    let conv = conv.filter(|c| c.len() <= 64 && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'));
    show_main(&app);
    let w = main_window(&app).ok_or("no main window")?;
    let _ = w;
    app.emit_to("main", "famrev-open", OpenEvent { route, conv }).map_err(|e| e.to_string())
}

/// Start one program from the fixed list (Outlook, Excel, Word, Teams, Calculator). Windows only.
#[tauri::command]
fn open_program(name: String) -> Result<(), String> {
    let target = PROGRAMS.iter().find(|(k, _)| *k == name).map(|(_, v)| *v).ok_or("not an allowed program")?;
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", target])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = target;
        Err("opening programs works on Windows only".into())
    }
}

/// Open a web link in the person's default browser (Teams / Zoom links go on to their own app from there).
/// 2.0.2 (owner 2026-11-20: "in calendar the Join button is not working in the desktop app"): WebView2 ignores links
/// that ask for a new window (target=_blank), so the page asks the app instead. Only http(s) addresses on another
/// host than FAMREV, no user name / password in them, at most 4,000 characters.
/// 2.0.3 (owner 2026-11-22: "Join opens C:\\Users\\…\\Documents"): 2.0.2 started explorer.exe with the address, and
/// explorer.exe treats an argument it cannot read as a path (Teams join links are long, with %-codes) and opens the
/// Documents folder instead. Now Windows' own ShellExecuteW "open" gets the checked http(s) address: the default
/// browser opens it, as when a link is clicked anywhere else. No command shell, no program name from the page.
#[tauri::command]
fn open_link(url: String) -> Result<(), String> {
    let u = check_link(&url)?;
    #[cfg(target_os = "windows")]
    {
        shell_open(u.as_str())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = u;
        Err("opening links works on Windows only".into())
    }
}

#[cfg(target_os = "windows")]
fn shell_open(url: &str) -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
    let (verb, file) = (wide("open"), wide(url));
    // SAFETY: both strings are NUL-terminated UTF-16 buffers that live until the call returns; the other pointers
    // are null, which ShellExecuteW accepts (no owner window, no parameters, no working folder).
    let r = unsafe {
        ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL)
    };
    if r as isize > 32 {
        Ok(())
    } else {
        Err(format!("Windows could not open the link (code {})", r as isize))
    }
}

fn check_link(url: &str) -> Result<tauri::Url, String> {
    if url.len() > 4000 || url.chars().any(|c| c.is_control() || c == '"' || c == ' ') {
        return Err("not a web link".into());
    }
    let u = tauri::Url::parse(url).map_err(|_| "not a web link".to_string())?;
    if u.scheme() != "https" && u.scheme() != "http" {
        return Err("only web links (http / https) are opened".into());
    }
    if !u.username().is_empty() || u.password().is_some() || u.host_str().unwrap_or("").is_empty() {
        return Err("not a web link".into());
    }
    Ok(u)
}

/// One picture of the screen the quick box is on, as a PNG data URL. The quick box hides while it is taken, so it is
/// not in the picture. Only on the person's click in the page; nothing is saved on the PC.
#[tauri::command]
async fn capture_screen(app: AppHandle) -> Result<String, String> {
    let qb = app.get_webview_window("quickbox");
    let (cx, cy) = match &qb {
        Some(w) => {
            let p = w.outer_position().map_err(|e| e.to_string())?;
            let s = w.outer_size().map_err(|e| e.to_string())?;
            (p.x + s.width as i32 / 2, p.y + s.height as i32 / 2)
        }
        None => (0, 0),
    };
    if let Some(w) = &qb {
        let _ = w.hide();
    }
    std::thread::sleep(std::time::Duration::from_millis(220)); // let the window disappear from the screen
    let shot = (|| -> Result<String, String> {
        let monitors = xcap::Monitor::all().map_err(|e| e.to_string())?;
        let m = monitors
            .iter()
            .find(|m| {
                let (x, y) = (m.x().unwrap_or(0), m.y().unwrap_or(0));
                let (w, h) = (m.width().unwrap_or(0) as i32, m.height().unwrap_or(0) as i32);
                cx >= x && cx < x + w && cy >= y && cy < y + h
            })
            .or_else(|| monitors.iter().find(|m| m.is_primary().unwrap_or(false)))
            .or(monitors.first())
            .ok_or("no screen found")?;
        let img = m.capture_image().map_err(|e| e.to_string())?;
        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)))
    })();
    if let Some(w) = &qb {
        let _ = w.show();
        let _ = w.set_focus();
    }
    shot
}

#[tauri::command]
fn app_info(app: AppHandle) -> AppInfo {
    AppInfo { version: app.package_info().version.to_string(), server: SERVER.to_string(), os: std::env::consts::OS.to_string() }
}

// ------------------------------------------------------------------------------------------------- startup diagnostics
/// %LOCALAPPDATA%\FAMREV\desktop-startup.log: each start writes its steps here, and any startup error or panic.
fn log_path() -> std::path::PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("FAMREV").join("desktop-startup.log")
}

fn slog(msg: &str) {
    use std::io::Write;
    let p = log_path();
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = writeln!(f, "{t} [{} {}] {msg}", env!("CARGO_PKG_VERSION"), SERVER);
    }
}

/// Log the error and show it in a box, so a failed start is not silent (release builds have no console).
fn fail(msg: &str) {
    slog(&format!("FATAL: {msg}"));
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        let text = format!("FAMREV AI could not start.\n\n{msg}\n\nLog: {}", log_path().display());
        let w = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        let (t, c) = (w(&text), w("FAMREV AI"));
        unsafe {
            MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), MB_OK | MB_ICONERROR);
        }
    }
}

// ------------------------------------------------------------------------------------------------- app
pub fn run() {
    std::panic::set_hook(Box::new(|info| fail(&format!("panic: {info}"))));
    slog("start");
    let res = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            slog("second start: showing the running app");
            show_main(app)
        }))
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_denylist(&["quickbox"]) // the quick box always opens at the bottom centre
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, sc, ev| {
                    if *sc == hotkey() && ev.state() == ShortcutState::Pressed {
                        let _ = qb_toggle_impl(app);
                    }
                })
                .build(),
        )
        .manage(QbState::default())
        .invoke_handler(tauri::generate_handler![qb_show, qb_hide, qb_toggle, qb_resize, open_main, open_program, open_link, capture_screen, app_info])
        .setup(|app| {
            slog("setup: plugins loaded");
            let h = app.handle().clone();
            // main window: the local start page checks the server is reachable, then opens it
            WebviewWindowBuilder::new(&h, "main", WebviewUrl::App("index.html".into()))
                .title("FAMREV AI")
                .inner_size(1200.0, 820.0)
                .min_inner_size(480.0, 600.0)
                .center()
                .disable_drag_drop_handler() // files dragged onto FAMREV reach the page (attachments)
                .additional_browser_args(BROWSER_ARGS)
                .build()
                .map_err(|e| { slog(&format!("main window failed: {e}")); e })?;
            slog("setup: main window ok");
            quickbox(&h).map_err(|e| { slog(&format!("quick box failed: {e}")); e })?;
            slog("setup: quick box ok"); // made now (hidden), so the hotkey shows it at once
            if let Err(e) = h.global_shortcut().register(hotkey()) {
                slog(&format!("Ctrl+Alt+Space is taken by another program: {e}"));
            }
            let open = MenuItem::with_id(&h, "open", "Open FAMREV", true, None::<&str>)?;
            let qb = MenuItem::with_id(&h, "qb", "Quick box\tCtrl+Alt+Space", true, None::<&str>)?;
            let quit = MenuItem::with_id(&h, "quit", "Quit FAMREV", true, None::<&str>)?;
            let menu = Menu::with_items(&h, &[&open, &qb, &quit])?;
            TrayIconBuilder::with_id("famrev")
                .icon(h.default_window_icon().cloned().expect("app icon"))
                .tooltip("FAMREV AI")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, ev| match ev.id().as_ref() {
                    "open" => show_main(app),
                    "qb" => {
                        let _ = qb_show_impl(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                        let app = tray.app_handle();
                        match main_window(app) {
                            Some(w) if w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false) => {
                                let _ = w.hide();
                            }
                            _ => show_main(app),
                        }
                    }
                })
                .build(&h)
                .map_err(|e| { slog(&format!("tray failed: {e}")); e })?;
            slog("setup: tray ok, started");
            Ok(())
        })
        .on_window_event(|win, ev| {
            if let WindowEvent::CloseRequested { api, .. } = ev {
                // closing hides (main to the tray, the quick box out of sight); Quit is in the tray menu
                api.prevent_close();
                let _ = win.hide();
            }
        })
        .run(tauri::generate_context!());
    if let Err(e) = res {
        fail(&format!("{e}"));
    }
}


#[cfg(test)]
mod link_tests {
    use super::check_link;
    #[test]
    fn web_links_only() {
        assert!(check_link("https://teams.microsoft.com/l/meetup-join/19%3a1@thread.v2/0?context=%7b%7d&x=1").is_ok());
        assert!(check_link("http://example.com/a").is_ok());
        for bad in ["file:///C:/Windows/system32/calc.exe", "javascript:alert(1)", "ms-settings:", "msteams:/l/x",
                    "https://user:pw@example.com/", "https://exa mple.com", "https://x.com/\"\" & calc", "https://", ""] {
            assert!(check_link(bad).is_err(), "{bad}");
        }
    }
}
