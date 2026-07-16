---
name: verify
description: Build, launch, and drive Riko Launcher to verify changes at the GUI surface
---

# Verifying Riko Launcher

## Build / launch

- Rust check: `cargo check --workspace`. Windows cross-check: `cargo xwin check -p riko-core --target x86_64-pc-windows-msvc` (plain cargo check on that target fails on bzip2-sys needing lib.exe).
- Frontend check: `cd frontend && npx svelte-check --threshold error`.
- Run the app from the **repo root** (frontend/ and src-tauri/ are siblings): `GDK_BACKEND=x11 frontend/node_modules/.bin/tauri dev` (cargo-tauri is not installed globally; the CLI lives in frontend/node_modules). Run it in the background; the window is titled "Riko Launcher" and takes ~30–60s to appear on first build.

## Driving the GUI - gotchas

- Host is KDE Plasma on Wayland. **xdotool clicks/keys/mousemove do NOT work**: KWin gates synthetic input behind a libei prompt (`kwin_eis_prompter`) the user must approve. Don't waste time on it. No Xvfb/Xephyr installed.
- What DOES work via xdotool/XWayland: `search --name`, `windowactivate`, `windowsize`, `getwindowgeometry`, and ImageMagick `import -window $W shot.png` for screenshots.
- To navigate: the app is a hash router (`frontend/src/lib/router.svelte.ts`) served by Vite dev with HMR. Temporarily append `window.location.hash = "#/plugins";` (or any route: /friends, /stats, /doctor, /settings, /game/<id>) to the end of router.svelte.ts - Vite full-reloads the webview onto that page. `touch` the file to force another reload (backend Tauri commands re-run on reload). **Revert the edit when done.**
- To see below the fold, resize taller: `xdotool windowsize $W 1200 1440`.
- Shutdown: `pkill -f riko-launcher; pkill -f "tauri dev"; pkill -f vite`.

## What's drivable

- All read-only pages (Library, Plugins, Stats, Doctor, Friends) render real backend data via list/get Tauri commands on page load - good verification surface without input.
- Game launch (`launch_game`) hits playvortex.io with the user's real account and spawns wine - do not drive it autonomously; note it as not exercised.
