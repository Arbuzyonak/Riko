use crate::events::TauriSink;
use crate::state::AppState;
use riko_core::doctor::{CheckResult, FixKind};
use riko_core::RikoError;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn run_doctor(state: State<'_, AppState>) -> Result<Vec<CheckResult>, RikoError> {
    let cfg = state.config.read().await.clone();
    tokio::task::spawn_blocking(move || riko_core::doctor::run_checks(&cfg))
        .await
        .map_err(|e| RikoError::Other(e.to_string()))
}

#[tauri::command]
pub async fn apply_fix(
    app: AppHandle,
    state: State<'_, AppState>,
    fix: FixKind,
) -> Result<(), RikoError> {
    match fix {
        FixKind::RegisterUri => riko_core::platform::register_uri(),
        FixKind::RunUpdate => {
            let cfg = state.config.read().await.clone();
            let sink = TauriSink::new(app, "update://progress");
            riko_core::updater::download_vortex(
                &cfg.paths.vortex_exe,
                cfg.auth.session_token.as_deref(),
                &sink,
            )
            .await
        }
        FixKind::Command { .. } | FixKind::RunSetup | FixKind::RunLogin => Err(RikoError::Other(
            "this fix is applied from the interface".to_string(),
        )),
    }
}
