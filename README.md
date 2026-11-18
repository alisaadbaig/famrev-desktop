# FAMREV AI: Windows desktop app (Tauri 2)

A small native app (~10 MB) that opens FAMREV (https://chat.famrev.ai) in its own window, and adds what a browser tab
can't:

- **Quick box, always on top.** **Ctrl+Alt+Space** from any program opens a small FAMREV box at the bottom of the
  screen the mouse is on. It stays above Outlook, Excel and everything else, also when the FAMREV window is minimized
  or hidden. Ask, drop files on it, add the open e-mail, take a screenshot, open Email / Phone / Fax, start Outlook /
  Excel / Word / Teams. Press the hotkey again (or Esc) to hide it. It needs the `quick_box` switch in FAMREV
  (v151.4.44 or later).
- **System tray:** left-click shows / hides FAMREV; right-click: Open FAMREV · Quick box · Quit. Closing the window
  hides it to the tray (FAMREV keeps running, like Teams); Quit is in the tray menu.
- **One copy:** starting the app again brings the running one forward.
- **Remembers** the main window's size and place.
- **Files dragged from Explorer** reach FAMREV (attachments, the quick box).
- A branded "Can't reach FAMREV" screen with Retry when the server can't be reached (VPN / network).

## What the FAMREV pages may ask the app to do

`src-tauri/capabilities/remote.json`: only pages from **https://chat.famrev.ai** and **https://uat.famrev.ai**, and only:
show / hide / resize the quick box, bring the main window to a FAMREV page (chat, email, phone, fax, notes, files),
start a program from a FIXED list (Outlook, Excel, Word, Teams, Calculator: never a path or arguments from the page),
take one screenshot of the screen the quick box is on (only when the person clicks Take screenshot; the box hides
while it is taken; nothing is saved on the PC), and read the app version. No file access, no shell, no other windows.

## Build the installers: GitHub Actions (no Windows machine needed)

1. Push this folder to the GitHub repository.
2. Actions → "Build FAMREV AI Desktop (Windows)" → Run workflow (or push a tag `v2.0.0`).
3. Each run makes TWO installers (Artifacts):
   - `famrev-ai-windows-production`: **FAMREV AI** → chat.famrev.ai
   - `famrev-ai-windows-uat`: **FAMREV AI UAT** → uat.famrev.ai (for QA; installs next to the normal app)

   Each has an `.msi` (Intune / Group Policy / SCCM) and an `.exe` (per-user install).

On a Windows 11 PC instead: install Rust (https://rustup.rs) and Node 20+, then `npm install`, then
`npm run build` (production) or `set FAMREV_URL=https://uat.famrev.ai && npm run build:uat` (UAT).

## Before the wide rollout

- **Code signing:** unsigned installers get Windows SmartScreen "unknown publisher" warnings and may be blocked by IT.
  Buy a code-signing certificate; it goes into the workflow as encrypted repository secrets.
- **Automatic updates** (Tauri updater): needs a signing key pair (`npx tauri signer generate`; the private key stays a
  repository secret) and a small `latest.json` on the FAMREV server. Planned as the next step, so the 1000 PCs update
  themselves.

## Changing things

- Server: the `FAMREV_URL` environment variable at build time (default https://chat.famrev.ai).
- Hotkey: `hotkey()` in `src-tauri/src/lib.rs` (Ctrl+Alt+Space; not Ctrl+Shift+Space, which the web page uses, and
  not Alt+Space, which Windows and Microsoft Copilot use).
- Programs the quick box may start: `PROGRAMS` in `src-tauri/src/lib.rs`.

Version 2.0.1 (2026-11-18): fixes 2.0.0 closing by itself right after it opened on Windows. WebView2 needs every window
of the app started with the same browser arguments; the quick box had none. Both windows now share them. Also:
- a quick box or tray icon that can't be made no longer stops the app;
- start-up problems and crashes go to `%LOCALAPPDATA%\FAMREV\desktop.log`;
- every build starts the app on Windows for 25 s and fails if it closes or a window is missing.

Version 2.0.0 (2026-11-18): Tauri 1 → 2; quick box window, global hotkey, tray, close-to-tray, single instance, window
size memory, file drops reach the page, UAT build. Version 1.0.0 opened the window only (the hotkey, tray and
notifications described for it had not been written).
