# FAMREV AI — Windows Desktop App (Tauri)

A lean (~8 MB), claude.ai-style desktop app for FAMREV AI. It loads your live
web UI (https://chat.famrev.ai) in a clean native window, and adds:

- 🪟 Dedicated app window (no browser chrome) + Start-menu / taskbar presence
- ⌨️  Global hotkey **Ctrl+Shift+Space** — summon/hide from anywhere
- 📍 System tray — left-click to toggle, right-click for menu
- ✖️  Close hides to tray (stays resident, like claude.ai) — Quit from tray to exit
- 🔔 Native notifications support

## How to build the .exe — TWO options

### Option A — GitHub Actions (no Windows machine needed) ⭐ recommended
1. Push this whole folder to your GitHub repo.
2. Go to the repo → **Actions** tab → enable workflows if prompted.
3. Either:
   - push a tag:  `git tag v1.0.0 && git push --tags`   (auto-builds), or
   - Actions tab → "Build FAMREV AI Desktop" → **Run workflow**.
4. When it finishes, open the run → **Artifacts** → download `famrev-ai-windows`.
   Inside: a `.msi` and an `.exe` installer. That's your app.

### Option B — build on a Windows 11 machine
Prereqs (one-time): install Rust (https://rustup.rs), Node 20, and the
Microsoft C++ Build Tools + WebView2 (WebView2 ships with Win11).
Then:
```
npm install
npm run build
```
Installers appear in `src-tauri/target/release/bundle/` (msi/ and nsis/).

## Deploying to your 1000 users
- The `.msi` is ideal for push-deployment via Intune / Group Policy / SCCM.
- The `.exe` (NSIS) is a per-user installer for manual installs.

## IMPORTANT — code signing (do before wide rollout)
Unsigned installers trigger Windows SmartScreen ("unknown publisher") warnings
and may be blocked by IT policy. Get a code-signing certificate (~$200-400/yr),
then sign the .msi/.exe. I can wire automatic signing into the GitHub workflow
once you have the cert (it goes in as encrypted repo secrets).

## Changing the URL
Edit `src-tauri/tauri.conf.json` → `tauri.windows[0].url`.

## Changing the hotkey
Edit `src-tauri/src/main.rs` → the `"CmdOrControl+Shift+Space"` string.
