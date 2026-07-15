use crate::config::Config;
use crate::{RikoError, VORTEX_BASE};
use serde::{Deserialize, Serialize};

const DISCOVERY_WINDOW: u32 = 5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub creator: Option<String>,
}

pub async fn fetch_all(token: &str) -> Result<Vec<Game>, RikoError> {
    let client = reqwest::Client::new();
    let mut games = Vec::new();
    let mut start = 1u32;
    loop {
        let ids: Vec<u32> = (start..start + DISCOVERY_WINDOW).collect();
        let results = futures_util::future::join_all(
            ids.iter().map(|id| fetch_game(&client, token, *id)),
        )
        .await;
        let mut done = false;
        for result in results {
            match result? {
                Some(game) => games.push(game),
                None => {
                    done = true;
                    break;
                }
            }
        }
        if done {
            break;
        }
        start += DISCOVERY_WINDOW;
    }
    save_cache(&games);
    Ok(games)
}

async fn fetch_game(
    client: &reqwest::Client,
    token: &str,
    id: u32,
) -> Result<Option<Game>, RikoError> {
    let resp = client
        .get(format!("{VORTEX_BASE}/api/games/{id}"))
        .header("Cookie", format!("session_token={token}"))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let body: serde_json::Value = resp.json().await?;
    let name = match body.get("name").and_then(|v| v.as_str()) {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => return Ok(None),
    };

    let string_field = |keys: &[&str]| {
        keys.iter().find_map(|k| {
            body.get(*k)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
    };

    let creator = body
        .get("creator")
        .and_then(|c| {
            c.as_str()
                .map(str::to_string)
                .or_else(|| c.get("username").and_then(|u| u.as_str()).map(str::to_string))
        })
        .filter(|s| !s.is_empty());

    Ok(Some(Game {
        id,
        name,
        description: string_field(&["description"]),
        thumbnail_url: string_field(&["thumbnail_url", "thumbnail", "icon_url", "image_url"]),
        creator,
    }))
}

fn cache_path() -> std::path::PathBuf {
    Config::data_dir().join("games.json")
}

pub fn load_cached() -> Vec<Game> {
    std::fs::read_to_string(cache_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_cache(games: &[Game]) {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(json) = serde_json::to_string_pretty(games) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            std::fs::rename(&tmp, &path).ok();
        }
    }
}
