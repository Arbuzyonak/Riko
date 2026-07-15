use crate::state::AppState;
use riko_core::RikoError;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct AppStatus {
    pub logged_in: bool,
    pub username: Option<String>,
    pub setup_needed: bool,
    pub migrated_from_tempest: bool,
}

#[tauri::command]
pub async fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, RikoError> {
    let cfg = state.config.read().await;
    let plan = riko_core::platform::setup_plan(&cfg);
    Ok(AppStatus {
        logged_in: cfg.auth.session_token.is_some(),
        username: cfg.auth.username.clone(),
        setup_needed: plan.needs_setup(),
        migrated_from_tempest: state.migrated_from_tempest,
    })
}

#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<String, RikoError> {
    let token = riko_core::auth::login_direct(&username, &password).await?;
    let mut cfg = state.config.write().await;
    cfg.auth.session_token = Some(token);
    cfg.auth.username = Some(username.clone());
    cfg.save()?;
    Ok(username)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), RikoError> {
    let mut cfg = state.config.write().await;
    riko_core::auth::logout(&mut cfg)
}
