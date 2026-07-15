use crate::state::AppState;
use riko_core::friends::Friend;
use riko_core::RikoError;
use tauri::State;

#[tauri::command]
pub async fn get_friends(state: State<'_, AppState>) -> Result<Vec<Friend>, RikoError> {
    let token = {
        let cfg = state.config.read().await;
        cfg.auth.session_token.clone().ok_or(RikoError::NotLoggedIn)?
    };
    riko_core::friends::fetch_friends(&token).await
}
