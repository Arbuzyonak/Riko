use crate::events::TauriSink;
use crate::state::AppState;
use riko_core::marketplace::CatalogEntry;
use riko_core::plugin::PluginInfo;
use riko_core::RikoError;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct MarketplaceEntry {
    #[serde(flatten)]
    pub entry: CatalogEntry,
    pub installed: bool,
}

#[tauri::command]
pub async fn list_marketplace(
    state: State<'_, AppState>,
) -> Result<Vec<MarketplaceEntry>, RikoError> {
    let cfg = state.config.read().await.clone();
    let url = riko_core::marketplace::catalog_url(&cfg);
    let entries = riko_core::marketplace::fetch_catalog(&url).await?;
    let installed: Vec<String> = riko_core::plugin::list(&cfg)
        .into_iter()
        .filter(|p| p.installed)
        .map(|p| p.name)
        .collect();
    Ok(entries
        .into_iter()
        .map(|entry| MarketplaceEntry {
            installed: installed.contains(&entry.name),
            entry,
        })
        .collect())
}

#[tauri::command]
pub async fn install_marketplace_plugin(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<PluginInfo, RikoError> {
    let cfg = state.config.read().await.clone();
    let url = riko_core::marketplace::catalog_url(&cfg);
    let entry = riko_core::marketplace::fetch_catalog(&url)
        .await?
        .into_iter()
        .find(|e| e.name == name)
        .ok_or_else(|| RikoError::Plugin(format!("'{name}' is not in the catalog")))?;

    let sink = TauriSink::new(app, "plugin://progress");
    riko_core::marketplace::download_and_extract(&entry, &sink).await?;
    riko_core::plugin::install(&cfg, &name, &sink).await
}
