use crate::config::{Config, StoredAccount};
use crate::{net, RikoError, USER_AGENT, VORTEX_BASE};

pub async fn login_direct(username: &str, password: &str) -> Result<String, RikoError> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
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

pub fn upsert_account(cfg: &mut Config, username: &str, token: &str) {
    match cfg
        .accounts
        .iter_mut()
        .find(|a| a.username.eq_ignore_ascii_case(username))
    {
        Some(account) => {
            account.username = username.to_string();
            account.session_token = token.to_string();
        }
        None => cfg.accounts.push(StoredAccount {
            username: username.to_string(),
            session_token: token.to_string(),
        }),
    }
    cfg.auth.username = Some(username.to_string());
    cfg.auth.session_token = Some(token.to_string());
}

pub fn switch_account(cfg: &mut Config, username: &str) -> Result<(), RikoError> {
    let account = cfg
        .accounts
        .iter()
        .find(|a| a.username.eq_ignore_ascii_case(username))
        .ok_or_else(|| RikoError::Auth(format!("no saved account named '{username}'")))?;
    cfg.auth.username = Some(account.username.clone());
    cfg.auth.session_token = Some(account.session_token.clone());
    Ok(())
}

pub fn remove_account(cfg: &mut Config, username: &str) {
    cfg.accounts
        .retain(|a| !a.username.eq_ignore_ascii_case(username));
    if cfg
        .auth
        .username
        .as_deref()
        .is_some_and(|active| active.eq_ignore_ascii_case(username))
    {
        match cfg.accounts.first() {
            Some(next) => {
                cfg.auth.username = Some(next.username.clone());
                cfg.auth.session_token = Some(next.session_token.clone());
            }
            None => {
                cfg.auth.username = None;
                cfg.auth.session_token = None;
            }
        }
    }
}

pub fn logout(cfg: &mut Config) -> Result<(), RikoError> {
    let active = cfg.auth.username.clone();
    match active {
        Some(username) => remove_account(cfg, &username),
        None => {
            cfg.auth.session_token = None;
            cfg.auth.username = None;
        }
    }
    cfg.save()
}

pub async fn validate_session(token: &str) -> Result<bool, RikoError> {
    let resp = net::send_retrying(
        || {
            net::shared()
                .get(format!("{VORTEX_BASE}/api/games/1"))
                .header("Cookie", format!("session_token={token}"))
        },
        3,
    )
    .await?;
    let status = resp.status();
    Ok(!matches!(status.as_u16(), 401 | 403))
}

pub async fn get_play_uri(session_token: &str, game_id: u32) -> Result<String, RikoError> {
    let resp = net::shared()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_rotation_flow() {
        let mut cfg = Config::default();
        upsert_account(&mut cfg, "alice", "tok-a");
        upsert_account(&mut cfg, "bob", "tok-b");
        assert_eq!(cfg.accounts.len(), 2);
        assert_eq!(cfg.auth.username.as_deref(), Some("bob"));

        switch_account(&mut cfg, "alice").unwrap();
        assert_eq!(cfg.auth.session_token.as_deref(), Some("tok-a"));

        upsert_account(&mut cfg, "ALICE", "tok-a2");
        assert_eq!(cfg.accounts.len(), 2);
        assert_eq!(cfg.auth.session_token.as_deref(), Some("tok-a2"));

        remove_account(&mut cfg, "alice");
        assert_eq!(cfg.auth.username.as_deref(), Some("bob"));
        assert_eq!(cfg.auth.session_token.as_deref(), Some("tok-b"));

        remove_account(&mut cfg, "bob");
        assert!(cfg.auth.username.is_none());
        assert!(cfg.auth.session_token.is_none());
        assert!(switch_account(&mut cfg, "nobody").is_err());
    }
}
