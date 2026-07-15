pub mod build;
pub mod builtin;
pub mod manifest;

use crate::config::Config;
use crate::progress::ProgressSink;
use crate::RikoError;
use manifest::{EnvValue, PluginKind, PluginManifest};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    pub sandbox: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: PluginKind,
    pub platforms: Vec<String>,
    pub builtin: bool,
    pub installed: bool,
    pub built: bool,
    pub enabled: bool,
    pub supported: bool,
    pub build_command: Option<String>,
    pub missing_requirement: Option<String>,
}

pub fn plugins_dir() -> PathBuf {
    Config::data_dir().join("plugins")
}

fn load_dir_manifest(dir: &Path, name: &str) -> Option<PluginManifest> {
    if dir.join("plugin.toml").exists() {
        manifest::load(dir).ok()
    } else {
        builtin::manifest_for(name)
    }
}

fn info_from(cfg: &Config, m: &PluginManifest, dir: Option<&Path>) -> PluginInfo {
    let name = m.plugin.name.clone();
    let built = dir.map(|d| build::artifacts_present(d, m)).unwrap_or(false);
    PluginInfo {
        enabled: cfg.plugins.enabled.contains(&name),
        builtin: builtin::get(&name).is_some(),
        installed: dir.is_some(),
        built,
        supported: manifest::supported_on_current_platform(m),
        missing_requirement: builtin::manifest_for(&name)
            .map(|embedded| manifest::missing_requirement(&embedded))
            .unwrap_or_else(|| manifest::missing_requirement(m)),
        version: m.plugin.version.clone(),
        description: m.plugin.description.clone(),
        kind: m.plugin.kind,
        platforms: m.plugin.platforms.clone(),
        build_command: m.build.as_ref().map(|b| b.command.clone()),
        name,
    }
}

pub fn list(cfg: &Config) -> Vec<PluginInfo> {
    let mut infos: Vec<PluginInfo> = vec![];
    let mut seen: Vec<String> = vec![];
    let dir = plugins_dir();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if let Some(m) = load_dir_manifest(&path, &name)
                && m.plugin.name == name
            {
                seen.push(name);
                infos.push(info_from(cfg, &m, Some(&path)));
            }
        }
    }
    for b in builtin::BUILTINS {
        if !seen.iter().any(|s| s == b.name)
            && let Some(m) = builtin::manifest_for(b.name)
        {
            infos.push(info_from(cfg, &m, None));
        }
    }
    infos.sort_by(|a, b| a.name.cmp(&b.name));
    infos
}

pub async fn install(
    cfg: &Config,
    name: &str,
    sink: &dyn ProgressSink,
) -> Result<PluginInfo, RikoError> {
    let dir = plugins_dir().join(name);
    let dir = if dir.join("plugin.toml").exists() || builtin::get(name).is_none() {
        if !dir.exists() {
            return Err(RikoError::Plugin(format!("plugin '{name}' is not installed")));
        }
        dir
    } else {
        builtin::install_files(name)?
    };
    let m = manifest::load(&dir)?;
    build::build(&dir, &m, sink).await?;
    Ok(info_from(cfg, &m, Some(&dir)))
}

pub fn import(cfg: &Config, source: &Path) -> Result<PluginInfo, RikoError> {
    let m = manifest::load(source)?;
    let name = m.plugin.name.clone();
    let source_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if source_name != name {
        return Err(RikoError::Plugin(format!(
            "folder name '{source_name}' must match plugin name '{name}'"
        )));
    }
    let dest = plugins_dir().join(&name);
    if dest.exists() {
        return Err(RikoError::Plugin(format!("plugin '{name}' already exists")));
    }
    copy_dir_recursive(source, &dest)?;
    Ok(info_from(cfg, &m, Some(&dest)))
}

pub fn remove(name: &str) -> Result<(), RikoError> {
    let dir = plugins_dir().join(name);
    if !dir.is_dir() {
        return Err(RikoError::Plugin(format!("plugin '{name}' is not installed")));
    }
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

pub fn set_enabled(cfg: &mut Config, name: &str, game_id: Option<u32>, enabled: Option<bool>) {
    match game_id {
        None => {
            cfg.plugins.enabled.retain(|n| n != name);
            if enabled == Some(true) {
                cfg.plugins.enabled.push(name.to_string());
            }
        }
        Some(id) => {
            let key = id.to_string();
            let entry = cfg.plugins.per_game.entry(key.clone()).or_default();
            entry.enabled.retain(|n| n != name);
            entry.disabled.retain(|n| n != name);
            match enabled {
                Some(true) => entry.enabled.push(name.to_string()),
                Some(false) => entry.disabled.push(name.to_string()),
                None => {}
            }
            if entry.enabled.is_empty() && entry.disabled.is_empty() {
                cfg.plugins.per_game.remove(&key);
            }
        }
    }
}

pub fn resolve_env(cfg: &Config, game_id: Option<u32>) -> ResolvedPluginEnv {
    let mut resolved = ResolvedPluginEnv::default();
    let per_game = game_id.and_then(|id| cfg.plugins.per_game.get(&id.to_string()));

    let dir = plugins_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return resolved;
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    for plugin_dir in dirs {
        let Some(name) = plugin_dir.file_name().and_then(|n| n.to_str()).map(str::to_string)
        else {
            continue;
        };
        let Some(m) = load_dir_manifest(&plugin_dir, &name) else {
            continue;
        };
        if m.plugin.name != name || !manifest::supported_on_current_platform(&m) {
            continue;
        }

        let mut active = cfg.plugins.enabled.contains(&name);
        if let Some(pg) = per_game {
            if pg.disabled.contains(&name) {
                active = false;
            }
            if pg.enabled.contains(&name) {
                active = true;
            }
        }
        if !active || !build::artifacts_present(&plugin_dir, &m) {
            continue;
        }

        apply_manifest(&mut resolved, &m, &plugin_dir);
    }
    resolved
}

fn apply_manifest(resolved: &mut ResolvedPluginEnv, m: &PluginManifest, dir: &Path) {
    let expand = |value: &str| value.replace("${PLUGIN_DIR}", &dir.to_string_lossy());

    for (key, value) in &m.env {
        let final_value = match value {
            EnvValue::Fixed(v) => expand(v),
            EnvValue::Overridable { default } => {
                std::env::var(key).unwrap_or_else(|_| expand(default))
            }
        };
        resolved.env.insert(key.clone(), final_value);
    }

    for (key, append) in &m.env_append {
        let base = resolved
            .env
            .get(key)
            .cloned()
            .or_else(|| std::env::var(key).ok())
            .unwrap_or_default();
        let merged = merge_append(&base, &append.separator, &append.values);
        resolved.env.insert(key.clone(), merged);
    }

    if let Some(layer) = &m.vulkan_layer
        && dir.join(&layer.manifest).exists()
    {
        resolved.vulkan_layer_dirs.push(dir.to_path_buf());
    }

    if let Some(binary) = &m.binary
        && binary.run_after_launch
    {
        let path = dir.join(&binary.entrypoint);
        if path.is_file() {
            resolved.sidecars.push(Sidecar {
                path,
                delay_secs: binary.delay_secs,
                sandbox: binary.sandbox,
            });
        }
    }
}

fn merge_append(base: &str, separator: &str, values: &[String]) -> String {
    let mut parts: Vec<String> = if base.is_empty() {
        vec![]
    } else {
        base.split(separator).map(str::to_string).collect()
    };
    for value in values {
        let key_part = value.split('=').next().unwrap_or(value);
        if !parts.iter().any(|p| p.contains(key_part)) {
            parts.push(value.clone());
        }
    }
    parts.join(separator)
}

pub(crate) fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<(), RikoError> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_append_matches_tempest_dxvk_semantics() {
        let values = vec![
            "dxvk.enableAsync=true".to_string(),
            "dxvk.numCompilerThreads=2".to_string(),
        ];
        assert_eq!(
            merge_append("", ",", &values),
            "dxvk.enableAsync=true,dxvk.numCompilerThreads=2"
        );
        assert_eq!(
            merge_append("dxvk.hud=fps", ",", &values),
            "dxvk.hud=fps,dxvk.enableAsync=true,dxvk.numCompilerThreads=2"
        );
        assert_eq!(
            merge_append("dxvk.enableAsync=false", ",", &values),
            "dxvk.enableAsync=false,dxvk.numCompilerThreads=2"
        );
    }

    #[test]
    fn set_enabled_global_and_per_game() {
        let mut cfg = Config::default();
        set_enabled(&mut cfg, "fps-unlocker", None, Some(true));
        assert!(cfg.plugins.enabled.contains(&"fps-unlocker".to_string()));
        set_enabled(&mut cfg, "fps-unlocker", Some(4), Some(false));
        assert!(cfg.plugins.per_game["4"].disabled.contains(&"fps-unlocker".to_string()));
        set_enabled(&mut cfg, "fps-unlocker", Some(4), None);
        assert!(!cfg.plugins.per_game.contains_key("4"));
        set_enabled(&mut cfg, "fps-unlocker", None, Some(false));
        assert!(cfg.plugins.enabled.is_empty());
    }

    #[test]
    fn builtin_manifests_resolve_expected_env() {
        let m = builtin::manifest_for("vortex-optim").unwrap();
        let mut resolved = ResolvedPluginEnv::default();
        let dir = std::env::temp_dir();
        apply_manifest(&mut resolved, &m, &dir);
        assert_eq!(resolved.env.get("DXVK_STATE_CACHE").map(String::as_str), Some("1"));
        assert_eq!(resolved.env.get("mesa_glthread").map(String::as_str), Some("true"));
        assert!(resolved.env["DXVK_CONFIG"].contains("dxvk.enableAsync=true"));
    }

    #[test]
    fn env_only_builtins_resolve() {
        let dir = std::env::temp_dir();
        let mut resolved = ResolvedPluginEnv::default();
        apply_manifest(&mut resolved, &builtin::manifest_for("mangohud").unwrap(), &dir);
        assert_eq!(resolved.env.get("MANGOHUD").map(String::as_str), Some("1"));
        apply_manifest(&mut resolved, &builtin::manifest_for("fsr-upscale").unwrap(), &dir);
        assert_eq!(
            resolved.env.get("WINE_FULLSCREEN_FSR").map(String::as_str),
            Some("1")
        );
        apply_manifest(&mut resolved, &builtin::manifest_for("vkbasalt").unwrap(), &dir);
        assert_eq!(
            resolved.env.get("VKBASALT_CONFIG_FILE").map(String::as_str),
            Some(format!("{}/vkBasalt.conf", dir.display()).as_str())
        );
        apply_manifest(&mut resolved, &builtin::manifest_for("low-spec-mode").unwrap(), &dir);
        assert!(resolved.env["DXVK_CONFIG"].contains("dxvk.numCompilerThreads=2"));
        assert!(resolved.env["DXVK_CONFIG"].contains("dxvk.maxFrameLatency=1"));
    }
}
