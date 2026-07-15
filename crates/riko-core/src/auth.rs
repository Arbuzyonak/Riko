use crate::config::Config;
use crate::{RikoError, VORTEX_BASE};

pub async fn login_direct(username: &str, password: &str) -> Result<String, RikoError> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let resp = client
        .post(format!("{VORTEX_BASE}/login"))
        .form(&[
            ("username", username),
            ("password", password),
            ("fingerprint", ""),
            ("fp_token", ""),
        ])
        .send()
        .await?;

    let status = resp.status();
    if status.is_redirection() || status.is_success() {
        if let Some(cookie) = resp.cookies().find(|c| c.name() == "session_token") {
            return Ok(cookie.value().to_string());
        }
        return Err(RikoError::Auth(
            "server accepted login but set no session_token cookie".to_string(),
        ));
    }

    let body = resp.text().await.unwrap_or_default();
    let detail = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v["detail"].as_str().map(str::to_string))
        .unwrap_or_else(|| "invalid username or password".to_string());
    Err(RikoError::Auth(detail))
}

pub fn logout(cfg: &mut Config) -> Result<(), RikoError> {
    cfg.auth.session_token = None;
    cfg.auth.username = None;
    cfg.save()
}

pub async fn validate_session(token: &str) -> Result<bool, RikoError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{VORTEX_BASE}/api/games/1"))
        .header("Cookie", format!("session_token={token}"))
        .send()
        .await?;
    let status = resp.status();
    Ok(!matches!(status.as_u16(), 401 | 403))
}

pub async fn get_play_uri(session_token: &str, game_id: u32) -> Result<String, RikoError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{VORTEX_BASE}/games/{game_id}/play"))
        .header("Cookie", format!("session_token={session_token}"))
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        if matches!(status.as_u16(), 401 | 403) {
            return Err(RikoError::NotLoggedIn);
        }
        return Err(RikoError::Auth(format!(
            "failed to fetch play page: {status}"
        )));
    }

    let html = resp.text().await?;
    if let Some(start) = html.find("vortex://") {
        let uri = &html[start..];
        let end = uri
            .find(|c: char| c == '"' || c == '\'' || c.is_whitespace())
            .unwrap_or(uri.len());
        return Ok(uri[..end].to_string());
    }

    Err(RikoError::Auth(
        "could not find vortex:// URI in play page; the session token may be invalid or the site may have changed".to_string(),
    ))
}
