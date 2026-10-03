# FAMREV AI — Windows 11 Desktop App

A lean, native desktop client for FAMREV AI (built with Tauri 2). Loads your
web UI at https://chat.famrev.ai in a clean window with claude.ai-style features:

- 🪟 Clean native window (no browser chrome)
- ⌨️ Global hotkey: **Ctrl+Shift+Space** — summon/hide from anywhere
- 📌 System tray — left-click to toggle, right-click for menu; closing hides to tray
- 🔔 Native notifications
- 1️⃣ Single instance (re-launch focuses the existing window)
- ⬆️ Auto-update support (optional)
- 📦 ~8 MB installer, low memory (uses the Windows WebView2 runtime)

---

## Two ways to build the .exe

### Option A — GitHub Actions (recommended, no Windows toolchain needed)
1. Push this folder to a GitHub repo.
2. Go to **Actions → Build FAMREV AI Desktop → Run workflow** (or push a tag `v1.0.0`).
3. When it finishes, download the **famrev-ai-windows** artifact — it contains the
   `.exe` (NSIS installer) and `.msi`.

That's it — GitHub's Windows runners compile it for you.

### Option B — build on a Windows 11 machine
Prerequisites (one-time):
- Install **Rust**: https://rustup.rs
- Install **Node.js 20+**: https://nodejs.org
- Install **WebView2** (preinstalled on Win 11) and **Visual Studio Build Tools**
  (C++ workload): https://visualstudio.microsoft.com/downloads/

Then:
```
npm install
npm run build
```
Installers appear in:
- `src-tauri/target/release/bundle/nsis/FAMREV AI_1.0.0_x64-setup.exe`
- `src-tauri/target/release/bundle/msi/FAMREV AI_1.0.0_x64_en-US.msi`

---

## Before wide rollout (1000 users)

### 1. Replace the icons
The icons in `src-tauri/icons/` are placeholders (green "F"). Replace them with
your real logo, then rebuild. Easiest: `npm run tauri icon path/to/logo.png`
(generates all sizes automatically).

### 2. Code signing (important!)
Without a code-signing certificate, Windows SmartScreen warns users "unknown
publisher," and IT may block the installer. To sign:
- Buy a code-signing cert (OV ~$200-400/yr, or EV for instant SmartScreen trust).
- Add to `tauri.conf.json` under `bundle.windows`:
  ```json
  "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
  ```
- Rebuild. The installer and .exe will be signed.

### 3. Auto-update (optional but nice)
1. Generate signing keys: `npm run tauri signer generate -f famrev.key`
2. Put the **public** key in `tauri.conf.json` → `plugins.updater.pubkey`.
3. Keep the **private** key secret (add to GitHub secrets as
   `TAURI_SIGNING_PRIVATE_KEY`).
4. Host `updates.json` at `https://chat.famrev.ai/desktop/updates.json` describing
   the latest version + installer URL + signature. (Tauri docs: "Updater".)
Clients then auto-update silently.

### 4. Deploy to 1000 machines
- Push the `.msi` via **Intune / SCCM / Group Policy** for silent install, OR
- Host the `.exe` on an internal page and let users install it.

---

## Customizing
- **Change the hotkey:** edit `Code::Space` / modifiers in `src-tauri/src/main.rs`.
- **Change the URL:** edit `url` in `src-tauri/tauri.conf.json` (and `dist/index.html`).
- **Don't hide-to-tray on close:** remove the `on_window_event` block in `main.rs`.
- **Start with Windows:** add the `autostart` plugin (ask and I'll wire it).
