use riko_core::playtime::{PlaytimeEntry, SessionRecord};
use riko_core::RikoError;
use std::collections::HashMap;

#[tauri::command]
pub async fn get_playtime() -> Result<HashMap<u32, PlaytimeEntry>, RikoError> {
    Ok(riko_core::playtime::load())
}

#[tauri::command]
pub async fn get_sessions() -> Result<Vec<SessionRecord>, RikoError> {
    Ok(riko_core::playtime::load_sessions())
}
