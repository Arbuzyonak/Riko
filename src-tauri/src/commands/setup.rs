use crate::events::TauriSink;
use crate::state::AppState;
use riko_core::setup::{SetupPlan, SetupStep};
use riko_core::RikoError;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_setup_plan(state: State<'_, AppState>) -> Result<SetupPlan, RikoError> {
    let cfg = state.config.read().await;
    Ok(riko_core::platform::setup_plan(&cfg))
}

#[tauri::command]
pub async fn run_setup_step(
    app: AppHandle,
    state: State<'_, AppState>,
    step: SetupStep,
) -> Result<(), RikoError> {
    if state
        .setup_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(RikoError::Setup("another setup step is already running".to_string()));
    }
    let cfg = state.config.read().await.clone();
    let sink = TauriSink::new(app, "setup://progress");
    let result = riko_core::platform::execute_setup_step(step, &cfg, &sink).await;
    if result.is_ok() {
        cfg.save().ok();
    }
    state.setup_running.store(false, Ordering::SeqCst);
    result
}

#[tauri::command]
pub async fn uninstall_riko(state: State<'_, AppState>, app: AppHandle) -> Result<(), RikoError> {
    let cfg = state.config.read().await;
    riko_core::platform::uninstall(&cfg)?;
    app.exit(0);
    Ok(())
}
