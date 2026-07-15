use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct ResolvedPluginEnv {
    pub env: HashMap<String, String>,
    pub vulkan_layer_dirs: Vec<PathBuf>,
    pub sidecars: Vec<Sidecar>,
}

#[derive(Clone, Debug)]
pub struct Sidecar {
    pub path: PathBuf,
    pub delay_secs: u64,
}

pub fn plugins_dir() -> PathBuf {
    Config::data_dir().join("plugins")
}

fn plugin_dir(name: &str) -> PathBuf {
    plugins_dir().join(name)
}

pub fn installed_plugins() -> Vec<String> {
    let dir = plugins_dir();
    if !dir.is_dir() {
        return vec![];
    }
    let mut plugins = vec![];
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                plugins.push(name.to_string());
            }
        }
    }
    plugins.sort();
    plugins
}

pub fn resolve_env(_cfg: &Config, _game_id: Option<u32>) -> ResolvedPluginEnv {
    let mut resolved = ResolvedPluginEnv::default();
    for name in installed_plugins() {
        match name.as_str() {
            "fps-unlocker" => {
                let dir = plugin_dir("fps-unlocker");
                if dir.join("libVkLayer_vortstrap_present_mode.so").exists() {
                    resolved.vulkan_layer_dirs.push(dir);
                    resolved
                        .env
                        .insert("VORTSTRAP_FORCE_PRESENT".to_string(), "1".to_string());
                    resolved.env.insert(
                        "VORTSTRAP_PRESENT_MODE".to_string(),
                        getenv_or("VORTSTRAP_PRESENT_MODE", "0"),
                    );
                }
            }
            "vortex-optim" => {
                let binary = plugin_dir("vortex-optim").join("vortex-optim");
                if binary.is_file() {
                    resolved
                        .env
                        .insert("DXVK_STATE_CACHE".to_string(), "1".to_string());
                    resolved
                        .env
                        .insert("mesa_glthread".to_string(), "true".to_string());
                    resolved
                        .env
                        .insert("MESA_NO_DITHER".to_string(), "1".to_string());
                    let dxvk = getenv_or(
                        "DXVK_CONFIG",
                        "dxvk.enableAsync=true,dxvk.numCompilerThreads=2",
                    );
                    let merged = if dxvk.contains("dxvk.enableAsync") {
                        dxvk
                    } else {
                        format!("{dxvk},dxvk.enableAsync=true,dxvk.numCompilerThreads=2")
                    };
                    resolved.env.insert("DXVK_CONFIG".to_string(), merged);
                    resolved.sidecars.push(Sidecar {
                        path: binary,
                        delay_secs: 10,
                    });
                }
            }
            _ => {}
        }
    }
    resolved
}

fn getenv_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
