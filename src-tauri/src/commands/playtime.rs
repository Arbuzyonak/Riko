use riko_core::playtime::PlaytimeEntry;
use riko_core::RikoError;
use std::collections::HashMap;

#[tauri::command]
pub async fn get_playtime() -> Result<HashMap<u32, PlaytimeEntry>, RikoError> {
    Ok(riko_core::playtime::load())
}
