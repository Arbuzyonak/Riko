use riko_core::self_update::UpdateInfo;
use riko_core::RikoError;

#[tauri::command]
pub async fn check_riko_update() -> Result<Option<UpdateInfo>, RikoError> {
    riko_core::self_update::check().await
}
