mod commands;
mod state;

use state::AppState;

pub fn run() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }

    tauri::Builder::default()
        .manage(AppState::initialize())
        .invoke_handler(tauri::generate_handler![
            commands::auth::get_app_status,
            commands::auth::login,
            commands::auth::logout,
            commands::games::list_games,
        ])
        .run(tauri::generate_context!())
        .expect("error while running riko-launcher");
}
