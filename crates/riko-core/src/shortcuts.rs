use crate::config::Config;
use crate::games::Game;
use crate::RikoError;
use std::path::PathBuf;

pub async fn create_for_game(game: &Game) -> Result<PathBuf, RikoError> {
    let icon = download_icon(game).await;
    write_shortcut(game, icon.as_deref())
}

async fn download_icon(game: &Game) -> Option<PathBuf> {
    let url = game.thumbnail_url.as_ref()?;
    let icons_dir = Config::data_dir().join("icons");
    std::fs::create_dir_all(&icons_dir).ok()?;
    let path = icons_dir.join(format!("{}.png", game.id));
    let bytes = crate::net::downloader().get(url).send().await.ok()?.bytes().await.ok()?;
    std::fs::write(&path, &bytes).ok()?;
    Some(path)
}

#[cfg(target_os = "linux")]
fn write_shortcut(game: &Game, icon: Option<&std::path::Path>) -> Result<PathBuf, RikoError> {
    let exe = std::env::current_exe()?;
    let apps_dir = dirs::data_local_dir()
        .ok_or_else(|| RikoError::Config("cannot resolve data dir".to_string()))?
        .join("applications");
    std::fs::create_dir_all(&apps_dir)?;
    let path = apps_dir.join(format!("riko-game-{}.desktop", game.id));
    let name = game.name.replace('\n', " ");
    let icon_line = icon
        .map(|p| format!("Icon={}\n", p.display()))
        .unwrap_or_default();
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nComment=Play {name} on Vortex via Riko Launcher\nExec=\"{}\" --launch {}\n{icon_line}Terminal=false\nCategories=Game;\n",
        exe.display(),
        game.id
    );
    std::fs::write(&path, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(path)
}

#[cfg(target_os = "windows")]
fn write_shortcut(game: &Game, _icon: Option<&std::path::Path>) -> Result<PathBuf, RikoError> {
    let exe = std::env::current_exe()?;
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| RikoError::Config("cannot resolve desktop dir".to_string()))?;
    let path = desktop.join(format!("{}.bat", sanitize(&game.name)));
    let contents = format!("@echo off\r\nstart \"\" \"{}\" --launch {}\r\n", exe.display(), game.id);
    std::fs::write(&path, contents)?;
    Ok(path)
}

#[cfg(target_os = "macos")]
fn write_shortcut(game: &Game, _icon: Option<&std::path::Path>) -> Result<PathBuf, RikoError> {
    let exe = std::env::current_exe()?;
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| RikoError::Config("cannot resolve desktop dir".to_string()))?;
    let path = desktop.join(format!("{}.command", sanitize(&game.name)));
    let contents = format!("#!/bin/sh\nexec \"{}\" --launch {}\n", exe.display(), game.id);
    std::fs::write(&path, &contents)?;
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    Ok(path)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
        .collect()
}
