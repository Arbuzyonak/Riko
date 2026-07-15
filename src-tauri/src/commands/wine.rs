use crate::events::TauriSink;
use crate::state::AppState;
use riko_core::wine_versions::{AvailableWine, InstalledWine};
use riko_core::RikoError;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct WineVersions {
    pub installed: Vec<InstalledWine>,
    pub available: Vec<AvailableWine>,
    pub active_binary: String,
}

#[tauri::command]
pub async fn list_wine_versions(state: State<'_, AppState>) -> Result<WineVersions, RikoError> {
    let active_binary = state.config.read().await.wine.binary.clone();
    let available = riko_core::wine_versions::list_available()
        .await
        .unwrap_or_default();
    Ok(WineVersions {
        installed: riko_core::wine_versions::list_installed(),
        available,
        active_binary,
    })
}

#[tauri::command]
pub async fn install_wine_version(
    app: AppHandle,
    url: String,
) -> Result<InstalledWine, RikoError> {
    if !url.starts_with("https://github.com/Kron4ek/Wine-Builds/releases/download/") {
        return Err(RikoError::Other("unexpected wine build download URL".to_string()));
    }
    let sink = TauriSink::new(app, "wine://progress");
    riko_core::wine_versions::install(&url, &sink).await
}

#[tauri::command]
pub async fn remove_wine_version(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), RikoError> {
    let binary = riko_core::wine_versions::wine_dir()
        .join(&name)
        .join("bin/wine")
        .display()
        .to_string();
    riko_core::wine_versions::remove(&name)?;
    let mut cfg = state.config.write().await;
    if cfg.wine.binary == binary {
        cfg.wine.binary = "wine".to_string();
        cfg.save()?;
    }
    Ok(())
}
