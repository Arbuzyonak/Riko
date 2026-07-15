mod commands;
mod events;
mod state;

use state::AppState;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

pub fn run() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                window.unminimize().ok();
                window.set_focus().ok();
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::initialize())
        .setup(|app| {
            #[cfg(any(target_os = "linux", windows))]
            app.deep_link().register_all().ok();

            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                let urls: Vec<String> = event.urls().iter().map(|u| u.to_string()).collect();
                handle_uris(&handle, urls);
            });

            handle_uris(app.handle(), std::env::args().collect());

            #[cfg(debug_assertions)]
            if let Ok(route) = std::env::var("RIKO_START_ROUTE") {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    if let Some(window) = handle.get_webview_window("main") {
                        window
                            .eval(format!("window.location.hash = '{route}'"))
                            .ok();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::get_app_status,
            commands::auth::login,
            commands::auth::logout,
            commands::games::list_games,
            commands::launch::launch_game,
            commands::launch::stop_game,
            commands::launch::get_running_sessions,
            commands::setup::get_setup_plan,
            commands::setup::run_setup_step,
            commands::setup::uninstall_riko,
            commands::settings::get_config,
            commands::settings::update_config,
            commands::plugins::list_plugins,
            commands::plugins::install_plugin,
            commands::plugins::import_plugin,
            commands::plugins::remove_plugin,
            commands::plugins::set_plugin_enabled,
            commands::plugins::get_game_plugin_overrides,
            commands::doctor::run_doctor,
            commands::doctor::apply_fix,
            commands::playtime::get_playtime,
        ])
        .run(tauri::generate_context!())
        .expect("error while running riko-launcher");
}

fn handle_uris(app: &AppHandle, args: Vec<String>) {
    for arg in args {
        if let Some((game_id, token)) = riko_core::uri::parse_vortex_uri(&arg) {
            let uri = format!("vortex://play?game={game_id}&token={token}");
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                app.emit("nav://game", game_id).ok();
                let cfg = app.state::<AppState>().config.read().await.clone();
                if let Err(e) = commands::launch::spawn_game(&app, cfg, game_id, uri).await {
                    app.emit("game://launch-error", e.to_string()).ok();
                }
            });
            return;
        }
    }
}
