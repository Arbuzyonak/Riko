use crate::state::AppState;
use riko_core::games::Game;
use riko_core::RikoError;
use tauri::State;

#[tauri::command]
pub async fn list_games(state: State<'_, AppState>, refresh: bool) -> Result<Vec<Game>, RikoError> {
    if !refresh {
        let cached = riko_core::games::load_cached();
        if !cached.is_empty() {
            return Ok(cached);
        }
    }
    let token = {
        let cfg = state.config.read().await;
        cfg.auth.session_token.clone().ok_or(RikoError::NotLoggedIn)?
    };
    riko_core::games::fetch_all(&token).await
}
