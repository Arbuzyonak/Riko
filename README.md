<div align="center">

<img src=".github/assets/logo.png" width="132" alt="Riko logo" />

# Riko

**A fast, native launcher for [Vortex](https://playvortex.io) with plugins, a plugin marketplace, and Linux and Windows support.**

[![Build](https://github.com/Arbuzyonak/Riko/actions/workflows/build.yml/badge.svg)](https://github.com/Arbuzyonak/Riko/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/Arbuzyonak/Riko?color=8b5cf6)](https://github.com/Arbuzyonak/Riko/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows-8b5cf6)

</div>

## What is Riko

Riko is a desktop launcher for Vortex.
## Features

- **Game library** with live player counts, thumbnails, and per-game playtime.
- **One-click launch** into any Vortex game, with per-game launch profiles.
- **Plugin system** with a built-in FPS unlocker and a MangoHud overlay, plus a checksum-verified **marketplace** to install community plugins.
- **Wine version manager** (Linux): install and switch between Kron4ek builds per game.
- **Community shader cache** (opt-in): fetch a precompiled, checksum-verified shader cache for your GPU so the first minutes do not stutter.
- **Stats dashboard**: total playtime, launch counts, session history, and a 14-day activity chart.
- **Friends** with online status and avatars, plus `riko://join` invite links.
- **Doctor** panel that checks Wine, Vulkan, DXVK, the URI handler, and more, with one-click fixes.
- **Multiple accounts** with encrypted token storage and fast switching.
- **System tray**, close-to-tray, desktop shortcuts, and a `--launch <id>` CLI flag.
- **Discord Rich Presence** showing what you are playing.
- **Opt-in, anonymous** usage and crash reporting (off by default, never sends your account or logs).

## Install

Grab the latest build for your platform from the [Releases page](https://github.com/Arbuzyonak/Riko/releases/latest).

### Linux

Riko ships as `.deb` (Debian, Ubuntu, Pop!_OS) and `.rpm` (Fedora, openSUSE) packages.

**Debian / Ubuntu**

```sh
sudo apt install ./riko-launcher_*_amd64.deb
```

**Fedora**

```sh
sudo dnf install ./riko-launcher-*.x86_64.rpm
```

You also need **Wine** to run the Vortex client. Install it if you do not have it:

```sh
# Arch
sudo pacman -S wine
# Debian / Ubuntu
sudo apt install wine
# Fedora
sudo dnf install wine
```

Then launch **Riko** from your app menu, or run `riko-launcher` in a terminal. On first run, Riko installs a managed Wine prefix and downloads the Vortex client for you.

### Windows

1. Download the `.exe` (NSIS) installer, or the `.msi`, from the [latest release](https://github.com/Arbuzyonak/Riko/releases/latest).
2. Run the installer. If the [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/) is missing, the installer fetches it automatically.
3. Launch **Riko** from the Start menu.
4. On first run, Riko downloads the native Vortex client and registers the `vortex://` handler so the website can launch games through it.

## First run

1. Sign in with your Vortex account.
2. Let the Setup wizard finish (it downloads the Vortex client and, on Linux, prepares the Wine prefix).
3. Open a game and press **Play**.

Check the **Doctor** tab if anything looks off. It explains problems in plain language and offers a one-click fix for most of them.

## Plugins and the marketplace

Riko ships with two built-in plugins:

- **fps-unlocker** - removes the frame rate cap via a Vulkan implicit layer.
- **mangohud** - an in-game FPS, frametime, CPU, and GPU overlay.

Open **Plugins -> Marketplace** to browse and install community plugins. Every download is verified against a SHA-256 checksum before it is installed. See [`marketplace/README.md`](marketplace/README.md) to publish your own.

## Build from source

You need [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, and the Tauri system dependencies for your OS.

```sh
git clone https://github.com/Arbuzyonak/Riko
cd Riko

# install frontend dependencies
npm ci --prefix frontend

# run in development
frontend/node_modules/.bin/tauri dev

# or build installers
frontend/node_modules/.bin/tauri build
```

On Linux you will need `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, and `patchelf`. See the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for the full list.

## Configuration

Riko stores its config at `~/.config/riko/config.toml` (Linux) or `%APPDATA%\riko\config.toml` (Windows). Most settings have a toggle in the **Settings** tab, including telemetry and the community shader cache, which are both off by default.

## License

Riko is dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option. It is built on the archived [tempest](https://github.com/solomon-gleeson/tempest) CLI, which is available under the same terms. See [NOTICE.md](NOTICE.md) for attribution.
