mod dll;
mod doctor;
mod dxvk;
mod setup;
mod vkd3d;

pub use doctor::doctor_checks;
pub use setup::{execute_setup_step, setup_plan, uninstall, uri_handler_registered};

use crate::config::Config;
use crate::plugin::ResolvedPluginEnv;
use crate::RikoError;
use std::path::Path;
use std::process::Command;

pub const DESKTOP_FILE: &str = "io.riko.launcher.desktop";

pub fn register_uri() -> Result<(), RikoError> {
    let exe_path = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/riko-launcher"));

    let apps_dir = dirs::data_local_dir().unwrap_or_default().join("applications");
    std::fs::create_dir_all(&apps_dir)?;

    let contents = format!(
        "[Desktop Entry]\n\
         Name=Riko Launcher\n\
         Exec={} %u\n\
         Type=Application\n\
         MimeType=x-scheme-handler/vortex;\n\
         NoDisplay=true\n",
        exe_path.display()
    );
    std::fs::write(apps_dir.join(DESKTOP_FILE), contents)?;

    Command::new("xdg-mime")
        .args(["default", DESKTOP_FILE, "x-scheme-handler/vortex"])
        .status()
        .ok();
    Command::new("gio")
        .args(["mime", "x-scheme-handler/vortex", DESKTOP_FILE])
        .status()
        .ok();
    Command::new("update-desktop-database")
        .arg(&apps_dir)
        .status()
        .ok();

    Ok(())
}

pub fn unregister_uri() {
    let desktop = dirs::data_local_dir()
        .unwrap_or_default()
        .join("applications")
        .join(DESKTOP_FILE);
    if desktop.exists() {
        std::fs::remove_file(&desktop).ok();
        Command::new("update-desktop-database")
            .arg(desktop.parent().unwrap_or_else(|| Path::new(".")))
            .status()
            .ok();
    }
}

pub fn build_launch_command(cfg: &Config, uri: &str, plugin_env: &ResolvedPluginEnv) -> Command {
    let perf = &cfg.launcher;
    let use_gamemode = perf.use_gamemode && which::which("gamemoderun").is_ok();

    let mut cmd = if use_gamemode {
        let mut c = Command::new("gamemoderun");
        c.arg(&cfg.wine.binary);
        c
    } else {
        Command::new(&cfg.wine.binary)
    };

    cmd.env("WINEPREFIX", &cfg.paths.wine_prefix);
    cmd.env("WGPU_BACKEND", "vulkan");

    if perf.use_esync {
        cmd.env("WINEESYNC", "1");
    }
    if perf.use_fsync {
        cmd.env("WINEFSYNC", "1");
    }

    if perf.shader_cache {
        let cache = dirs::cache_dir()
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default() + "/.cache")
            })
            .join("vortex-shaders");
        std::fs::create_dir_all(&cache).ok();
        cmd.env("VKD3D_SHADER_CACHE_PATH", cache);
    }

    for (key, value) in &cfg.wine.env {
        cmd.env(key, value);
    }

    for (key, value) in &plugin_env.env {
        cmd.env(key, value);
    }

    if !plugin_env.vulkan_layer_dirs.is_empty() {
        let joined = plugin_env
            .vulkan_layer_dirs
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":");
        cmd.env("VK_ADD_IMPLICIT_LAYER_PATH", joined);
    }

    cmd.arg(&cfg.paths.vortex_exe);
    cmd.arg(uri);
    cmd
}

pub fn build_receiver_command(cfg: &Config, path: &Path) -> Command {
    let mut cmd = Command::new(&cfg.wine.binary);
    cmd.env("WINEPREFIX", &cfg.paths.wine_prefix);
    cmd.env("WINEDEBUG", "-all");
    cmd.arg(path);
    cmd
}

pub fn receiver_running() -> bool {
    Command::new("pgrep")
        .args(["-f", "receiver.exe"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub(crate) fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}
