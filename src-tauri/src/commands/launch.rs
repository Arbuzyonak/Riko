use crate::state::AppState;
use riko_core::launcher::{GameEvent, GameSession};
use riko_core::presence::PresenceCmd;
use riko_core::RikoError;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Serialize, Clone)]
struct EventPayload {
    game_id: u32,
    #[serde(flatten)]
    event: GameEvent,
}

#[derive(Serialize, Clone)]
struct LogLine {
    line: String,
    is_stderr: bool,
}

#[derive(Serialize, Clone)]
struct LogBatch {
    game_id: u32,
    lines: Vec<LogLine>,
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: u32,
    username: Option<String>,
) -> Result<u32, RikoError> {
    if state.sessions.lock().await.contains_key(&game_id) {
        return Err(RikoError::AlreadyRunning(game_id));
    }
    let (token, cfg) = {
        let cfg = state.config.read().await;
        let token = match &username {
            Some(name) => cfg
                .accounts
                .iter()
                .find(|a| a.username.eq_ignore_ascii_case(name))
                .map(|a| a.session_token.clone())
                .ok_or_else(|| RikoError::Auth(format!("no saved account named '{name}'")))?,
            None => cfg.auth.session_token.clone().ok_or(RikoError::NotLoggedIn)?,
        };
        (token, cfg.clone())
    };
    let uri = match riko_core::auth::get_play_uri(&token, game_id).await {
        Ok(uri) => uri,
        Err(RikoError::NotLoggedIn) => {
            app.emit("auth://expired", ()).ok();
            return Err(RikoError::NotLoggedIn);
        }
        Err(e) => return Err(e),
    };
    spawn_game(&app, cfg, game_id, uri).await
}

pub async fn spawn_game(
    app: &AppHandle,
    cfg: riko_core::Config,
    game_id: u32,
    uri: String,
) -> Result<u32, RikoError> {
    let state = app.state::<AppState>();
    let mut sessions = state.sessions.lock().await;
    if sessions.contains_key(&game_id) {
        return Err(RikoError::AlreadyRunning(game_id));
    }

    let cfg = cfg.effective_for_game(game_id);
    let plugin_env = riko_core::plugin::resolve_env(&cfg, Some(game_id));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let handle = riko_core::launcher::launch(&cfg, game_id, uri, plugin_env, tx).await?;
    let pid = handle.session.pid;
    let started_at_unix = handle.session.started_at.timestamp();
    sessions.insert(game_id, handle);
    drop(sessions);

    let game_name = riko_core::games::load_cached()
        .into_iter()
        .find(|g| g.id == game_id)
        .map(|g| g.name)
        .unwrap_or_else(|| format!("Game #{game_id}"));

    riko_core::overlay::write(&riko_core::overlay::OverlayState {
        game_id,
        game_name: game_name.clone(),
        started_at_unix,
        friends_online: 0,
    });

    if cfg.presence.enabled {
        state.presence.send(PresenceCmd::Playing {
            game_name,
            started_at_unix,
        });
    }

    if cfg.launcher.minimize_while_playing
        && let Some(window) = app.get_webview_window("main")
    {
        window.minimize().ok();
    }

    let app = app.clone();
    tokio::spawn(async move {
        let mut buf: Vec<GameEvent> = Vec::with_capacity(64);
        loop {
            let received = rx.recv_many(&mut buf, 64).await;
            if received == 0 {
                break;
            }
            let mut lines: Vec<LogLine> = Vec::new();
            for event in buf.drain(..) {
                match event {
                    GameEvent::Log { line, is_stderr } => lines.push(LogLine { line, is_stderr }),
                    other => {
                        if !lines.is_empty() {
                            app.emit(
                                "game://logs",
                                LogBatch {
                                    game_id,
                                    lines: std::mem::take(&mut lines),
                                },
                            )
                            .ok();
                        }
                        let exited = matches!(other, GameEvent::Exited { .. });
                        app.emit("game://event", EventPayload { game_id, event: other }).ok();
                        if exited {
                            let state = app.state::<AppState>();
                            state.sessions.lock().await.remove(&game_id);
                            if state.sessions.lock().await.is_empty() {
                                riko_core::overlay::clear();
                                state.presence.send(PresenceCmd::Idle);
                                if let Some(window) = app.get_webview_window("main") {
                                    window.unminimize().ok();
                                    window.show().ok();
                                    window.set_focus().ok();
                                }
                            }
                        }
                    }
                }
            }
            if !lines.is_empty() {
                app.emit("game://logs", LogBatch { game_id, lines }).ok();
            }
        }
    });

    Ok(pid)
}

#[tauri::command]
pub async fn stop_game(state: State<'_, AppState>, game_id: u32) -> Result<(), RikoError> {
    let handle = state.sessions.lock().await.remove(&game_id);
    match handle {
        Some(handle) => {
            handle.terminate();
            Ok(())
        }
        None => Err(RikoError::Other(format!("game {game_id} is not running"))),
    }
}

#[tauri::command]
pub async fn get_running_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<GameSession>, RikoError> {
    Ok(state
        .sessions
        .lock()
        .await
        .values()
        .map(|h| h.session.clone())
        .collect())
}
