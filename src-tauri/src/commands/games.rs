use crate::state::AppState;
use riko_core::games::{Game, GameStats};
use riko_core::RikoError;
use std::collections::HashMap;
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

#[tauri::command]
pub async fn create_shortcut(game_id: u32) -> Result<String, RikoError> {
    let game = riko_core::games::load_cached()
        .into_iter()
        .find(|g| g.id == game_id)
        .ok_or_else(|| RikoError::Other(format!("game {game_id} is not in the library cache")))?;
    let path = riko_core::shortcuts::create_for_game(&game).await?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub async fn get_game_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<u32, GameStats>, RikoError> {
    let token = {
        let cfg = state.config.read().await;
        cfg.auth.session_token.clone().ok_or(RikoError::NotLoggedIn)?
    };
    riko_core::games::fetch_stats(&token).await
}
