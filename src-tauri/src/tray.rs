use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().expect("bundled icon"))
        .tooltip("Riko Launcher")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(handle_menu_event)
        .build(app)?;
    Ok(())
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let open = MenuItem::with_id(app, "open", "Open Riko", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &separator])?;

    let games = riko_core::games::load_cached();
    let mut playtime: Vec<_> = riko_core::playtime::load().into_iter().collect();
    playtime.sort_by_key(|entry| std::cmp::Reverse(entry.1.last_played));
    let mut added = 0;
    for (game_id, _) in playtime {
        let Some(game) = games.iter().find(|g| g.id == game_id) else {
            continue;
        };
        let item = MenuItem::with_id(
            app,
            format!("game-{game_id}"),
            format!("Play {}", game.name),
            true,
            None::<&str>,
        )?;
        menu.append(&item)?;
        added += 1;
        if added == 3 {
            break;
        }
    }
    if added > 0 {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    menu.append(&quit)?;
    Ok(menu)
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "open" => show_main_window(app),
        "quit" => app.exit(0),
        id => {
            if let Some(game_id) = id.strip_prefix("game-").and_then(|v| v.parse::<u32>().ok()) {
                show_main_window(app);
                crate::handle_uris(app, vec!["--launch".to_string(), game_id.to_string()]);
            }
        }
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.show().ok();
        window.unminimize().ok();
        window.set_focus().ok();
    }
}
