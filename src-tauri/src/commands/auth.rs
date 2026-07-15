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
    riko_core::auth::upsert_account(&mut cfg, &username, &token);
    cfg.save()?;
    Ok(username)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), RikoError> {
    let mut cfg = state.config.write().await;
    riko_core::auth::logout(&mut cfg)
}

#[derive(Serialize)]
pub struct AccountView {
    pub username: String,
    pub active: bool,
}

fn account_views(cfg: &riko_core::Config) -> Vec<AccountView> {
    cfg.accounts
        .iter()
        .map(|a| AccountView {
            active: cfg
                .auth
                .username
                .as_deref()
                .is_some_and(|active| active.eq_ignore_ascii_case(&a.username)),
            username: a.username.clone(),
        })
        .collect()
}

#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> Result<Vec<AccountView>, RikoError> {
    let cfg = state.config.read().await;
    Ok(account_views(&cfg))
}

#[tauri::command]
pub async fn switch_account(
    state: State<'_, AppState>,
    username: String,
) -> Result<Vec<AccountView>, RikoError> {
    let mut cfg = state.config.write().await;
    riko_core::auth::switch_account(&mut cfg, &username)?;
    cfg.save()?;
    Ok(account_views(&cfg))
}

#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    username: String,
) -> Result<Vec<AccountView>, RikoError> {
    let mut cfg = state.config.write().await;
    riko_core::auth::remove_account(&mut cfg, &username);
    cfg.save()?;
    Ok(account_views(&cfg))
}
