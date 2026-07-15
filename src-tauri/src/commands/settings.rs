use crate::state::AppState;
use riko_core::config::LaunchOverrides;
use riko_core::RikoError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Serialize)]
pub struct ConfigView {
    pub username: Option<String>,
    pub has_session: bool,
    pub wine_binary: String,
    pub wine_env: HashMap<String, String>,
    pub filter_wine_noise: bool,
    pub auto_update: bool,
    pub use_esync: bool,
    pub use_fsync: bool,
    pub use_gamemode: bool,
    pub shader_cache: bool,
    pub minimize_while_playing: bool,
    pub presence_enabled: bool,
    pub wine_prefix: String,
    pub vortex_exe: String,
    pub log_file: String,
}

#[derive(Deserialize)]
pub struct ConfigPatch {
    pub wine_binary: Option<String>,
    pub wine_env: Option<HashMap<String, String>>,
    pub filter_wine_noise: Option<bool>,
    pub auto_update: Option<bool>,
    pub use_esync: Option<bool>,
    pub use_fsync: Option<bool>,
    pub use_gamemode: Option<bool>,
    pub shader_cache: Option<bool>,
    pub minimize_while_playing: Option<bool>,
    pub presence_enabled: Option<bool>,
}

fn view(cfg: &riko_core::Config) -> ConfigView {
    ConfigView {
        username: cfg.auth.username.clone(),
        has_session: cfg.auth.session_token.is_some(),
        wine_binary: cfg.wine.binary.clone(),
        wine_env: cfg.wine.env.clone(),
        filter_wine_noise: cfg.launcher.filter_wine_noise,
        auto_update: cfg.launcher.auto_update,
        use_esync: cfg.launcher.use_esync,
        use_fsync: cfg.launcher.use_fsync,
        use_gamemode: cfg.launcher.use_gamemode,
        shader_cache: cfg.launcher.shader_cache,
        minimize_while_playing: cfg.launcher.minimize_while_playing,
        presence_enabled: cfg.presence.enabled,
        wine_prefix: cfg.paths.wine_prefix.display().to_string(),
        vortex_exe: cfg.paths.vortex_exe.display().to_string(),
        log_file: cfg.paths.log_file.display().to_string(),
    }
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<ConfigView, RikoError> {
    let cfg = state.config.read().await;
    Ok(view(&cfg))
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    patch: ConfigPatch,
) -> Result<ConfigView, RikoError> {
    let mut cfg = state.config.write().await;
    if let Some(v) = patch.wine_binary {
        cfg.wine.binary = v;
    }
    if let Some(v) = patch.wine_env {
        cfg.wine.env = v;
    }
    if let Some(v) = patch.filter_wine_noise {
        cfg.launcher.filter_wine_noise = v;
    }
    if let Some(v) = patch.auto_update {
        cfg.launcher.auto_update = v;
    }
    if let Some(v) = patch.use_esync {
        cfg.launcher.use_esync = v;
    }
    if let Some(v) = patch.use_fsync {
        cfg.launcher.use_fsync = v;
    }
    if let Some(v) = patch.use_gamemode {
        cfg.launcher.use_gamemode = v;
    }
    if let Some(v) = patch.shader_cache {
        cfg.launcher.shader_cache = v;
    }
    if let Some(v) = patch.minimize_while_playing {
        cfg.launcher.minimize_while_playing = v;
    }
    if let Some(v) = patch.presence_enabled {
        cfg.presence.enabled = v;
        if !v {
            state.presence.send(riko_core::presence::PresenceCmd::Idle);
        }
    }
    cfg.save()?;
    Ok(view(&cfg))
}

#[tauri::command]
pub async fn get_launch_overrides(
    state: State<'_, AppState>,
    game_id: u32,
) -> Result<LaunchOverrides, RikoError> {
    let cfg = state.config.read().await;
    Ok(cfg
        .launch_overrides
        .get(&game_id.to_string())
        .cloned()
        .unwrap_or_default())
}

#[tauri::command]
pub async fn set_launch_overrides(
    state: State<'_, AppState>,
    game_id: u32,
    overrides: LaunchOverrides,
) -> Result<(), RikoError> {
    let mut cfg = state.config.write().await;
    if overrides.is_empty() {
        cfg.launch_overrides.remove(&game_id.to_string());
    } else {
        cfg.launch_overrides.insert(game_id.to_string(), overrides);
    }
    cfg.save()
}
