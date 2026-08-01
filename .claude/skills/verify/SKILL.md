---
name: verify
description: Build, launch, and drive Riko Launcher to verify changes at the GUI surface
---

# Verifying Riko Launcher
Riko runs on Linux and Windows - check which host you are on first, the GUI tooling differs.

## Build / launch

- Deps: `npm ci --prefix frontend`. Checks: `cargo check --workspace`, and
  `cd frontend && npx svelte-check --threshold error`.
- Run from the **repo root** (`frontend/` and `src-tauri/` are siblings) in the background;
  the "Riko Launcher" window takes ~30-60s to appear, longer on a cold Rust build.
  The Tauri CLI is not global - it lives in `frontend/node_modules/.bin/tauri`.
  - Linux: `GDK_BACKEND=x11 frontend/node_modules/.bin/tauri dev`. Needs
    `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `patchelf`. Cross-check with
    `cargo xwin check -p riko-core --target x86_64-pc-windows-msvc` (plain `cargo check` on
    that target fails on bzip2-sys needing `lib.exe`).
  - Windows: `frontend\node_modules\.bin\tauri dev`. Needs WebView2 (default with Edge) and
    a C toolchain for bzip2-sys/ring - a `cc-rs: failed to find tool "gcc.exe"` or missing
    `link.exe` means none is installed. Check `rustup show`; `-msvc` wants VS Build Tools
    with the C++ workload, `-gnu` wants mingw-w64. Both are multi-GB - ask before installing.

## Driving the GUI

Both platforms: the app is a hash router (`frontend/src/lib/router.svelte.ts`) behind Vite
HMR. Temporarily append `window.location.hash = "#/plugins";` (or /friends, /stats, /doctor,
/settings, /game/<id>) to the end of that file and Vite reloads the webview onto that page;
touch it again to re-run the backend commands. **Revert the edit when done.**

- **Linux** (KDE/Wayland): xdotool clicks/keys/mousemove do **not** work - KWin gates
  synthetic input behind a libei prompt. No Xvfb/Xephyr. What works: xdotool `search --name`,
  `windowactivate`, `windowsize` (e.g. `1200 1440` to see below the fold),
  `getwindowgeometry`, and `import -window $W shot.png` for screenshots.
  Stop with `pkill -f riko-launcher; pkill -f "tauri dev"; pkill -f vite`.
- **Windows**: no xdotool - use PowerShell + .NET (`System.Windows.Forms` /
  `System.Drawing`) for screenshots, and `user32.dll` P/Invoke (`MoveWindow`,
  `SetForegroundWindow`) on the handle from `Get-Process riko-launcher`. `SendKeys` only
  reaches the focused window, so prefer the router trick. Stop with `Stop-Process -Name
  riko-launcher`, plus the `tauri dev`/Vite `node` processes - check their command lines
  before killing.

## What's drivable

- Read-only pages (Library, Plugins, Stats, Doctor, Friends) render real backend data on
  page load - good verification surface without input.
- Game launch (`launch_game`) hits playvortex.io with the user's real account and, on Linux,
  spawns wine - do not drive it autonomously; note it as not exercised.
