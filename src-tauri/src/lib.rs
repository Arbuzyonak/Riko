mod commands;
mod events;
mod state;
mod tray;

use state::AppState;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

pub fn run() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            tray::show_main_window(app);
            handle_uris(app, argv);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::initialize())
        .setup(|app| {
            #[cfg(any(target_os = "linux", windows))]
            app.deep_link().register_all().ok();

            tray::setup(app.handle()).ok();

            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                let urls: Vec<String> = event.urls().iter().map(|u| u.to_string()).collect();
                handle_uris(&handle, urls);
            });

            handle_uris(app.handle(), std::env::args().collect());

            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = update_handle.state::<AppState>();
                let cfg = state.config.read().await.clone();
                if !cfg.launcher.auto_update {
                    return;
                }
                let Some(token) = cfg.auth.session_token.clone() else {
                    return;
                };
                let sink = events::TauriSink::new(update_handle.clone(), "update://progress");
                match riko_core::updater::update_if_stale(
                    &cfg.paths.vortex_exe,
                    Some(&token),
                    &sink,
                )
                .await
                {
                    Ok(true) => {
                        update_handle.emit("vortex://updated", ()).ok();
                    }
                    Ok(false) => {}
                    Err(e) => riko_core::logger::info(&format!("auto-update check failed: {e}")),
                }
            });

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
            commands::auth::list_accounts,
            commands::auth::switch_account,
            commands::auth::remove_account,
            commands::games::list_games,
            commands::games::get_game_stats,
            commands::games::create_shortcut,
            commands::friends::get_friends,
            commands::launch::launch_game,
            commands::launch::stop_game,
            commands::launch::get_running_sessions,
            commands::setup::get_setup_plan,
            commands::setup::run_setup_step,
            commands::setup::uninstall_riko,
            commands::settings::get_config,
            commands::settings::update_config,
            commands::settings::get_launch_overrides,
            commands::settings::set_launch_overrides,
            commands::plugins::list_plugins,
            commands::plugins::install_plugin,
            commands::plugins::import_plugin,
            commands::plugins::remove_plugin,
            commands::plugins::set_plugin_enabled,
            commands::plugins::get_game_plugin_overrides,
            commands::doctor::run_doctor,
            commands::doctor::apply_fix,
            commands::playtime::get_playtime,
            commands::playtime::get_sessions,
            commands::wine::list_wine_versions,
            commands::wine::install_wine_version,
            commands::wine::remove_wine_version,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().ok();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running riko-launcher");
}

pub(crate) fn handle_uris(app: &AppHandle, args: Vec<String>) {
    let mut args_iter = args.iter();
    while let Some(arg) = args_iter.next() {
        if let Some((game_id, token)) = riko_core::uri::parse_vortex_uri(arg) {
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
        if arg == "--launch"
            && let Some(game_id) = args_iter.next().and_then(|v| v.parse::<u32>().ok())
        {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                app.emit("nav://game", game_id).ok();
                let state = app.state::<AppState>();
                let (token, cfg) = {
                    let cfg = state.config.read().await;
                    (cfg.auth.session_token.clone(), cfg.clone())
                };
                let Some(token) = token else {
                    app.emit("game://launch-error", "not logged in".to_string()).ok();
                    return;
                };
                match riko_core::auth::get_play_uri(&token, game_id).await {
                    Ok(uri) => {
                        if let Err(e) = commands::launch::spawn_game(&app, cfg, game_id, uri).await
                        {
                            app.emit("game://launch-error", e.to_string()).ok();
                        }
                    }
                    Err(e) => {
                        app.emit("game://launch-error", e.to_string()).ok();
                    }
                }
            });
            return;
        }
    }
}
