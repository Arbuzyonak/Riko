use crate::events::TauriSink;
use crate::state::AppState;
use riko_core::config::PerGamePlugins;
use riko_core::plugin::PluginInfo;
use riko_core::RikoError;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, RikoError> {
    let cfg = state.config.read().await;
    Ok(riko_core::plugin::list(&cfg))
}

#[tauri::command]
pub async fn install_plugin(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<PluginInfo, RikoError> {
    let cfg = state.config.read().await.clone();
    let sink = TauriSink::new(app, "plugin://progress");
    riko_core::plugin::install(&cfg, &name, &sink).await
}

#[tauri::command]
pub async fn import_plugin(
    state: State<'_, AppState>,
    path: String,
) -> Result<PluginInfo, RikoError> {
    let cfg = state.config.read().await;
    riko_core::plugin::import(&cfg, std::path::Path::new(&path))
}

#[tauri::command]
pub async fn remove_plugin(state: State<'_, AppState>, name: String) -> Result<(), RikoError> {
    riko_core::plugin::remove(&name)?;
    let mut cfg = state.config.write().await;
    riko_core::plugin::set_enabled(&mut cfg, &name, None, Some(false));
    cfg.save()
}

#[tauri::command]
pub async fn set_plugin_enabled(
    state: State<'_, AppState>,
    name: String,
    game_id: Option<u32>,
    enabled: Option<bool>,
) -> Result<Vec<PluginInfo>, RikoError> {
    let mut cfg = state.config.write().await;
    riko_core::plugin::set_enabled(&mut cfg, &name, game_id, enabled);
    cfg.save()?;
    Ok(riko_core::plugin::list(&cfg))
}

#[tauri::command]
pub async fn get_game_plugin_overrides(
    state: State<'_, AppState>,
    game_id: u32,
) -> Result<PerGamePlugins, RikoError> {
    let cfg = state.config.read().await;
    Ok(cfg
        .plugins
        .per_game
        .get(&game_id.to_string())
        .cloned()
        .unwrap_or_default())
}
