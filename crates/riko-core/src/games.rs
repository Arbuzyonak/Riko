use crate::config::Config;
use crate::{RikoError, VORTEX_BASE};
use serde::{Deserialize, Serialize};

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
    let resp = client
        .get(format!("{VORTEX_BASE}/api/games"))
        .header("Cookie", format!("session_token={token}"))
        .send()
        .await?
        .error_for_status()?;
    let body: Vec<serde_json::Value> = resp.json().await?;
    let games: Vec<Game> = body.iter().filter_map(parse_game).collect();
    if games.is_empty() {
        return Err(RikoError::Other(
            "the games API returned no recognizable games; the site may have changed".to_string(),
        ));
    }
    save_cache(&games);
    Ok(games)
}

fn parse_game(body: &serde_json::Value) -> Option<Game> {
    let id = u32::try_from(body.get("id")?.as_u64()?).ok()?;
    let name = body
        .get("name")?
        .as_str()
        .filter(|n| !n.is_empty())?
        .to_string();

    let string_field = |keys: &[&str]| {
        keys.iter().find_map(|k| {
            body.get(*k)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
    };

    let thumbnail_url = Some(match string_field(&["thumbnail_version"]) {
        Some(version) => format!("{VORTEX_BASE}/assets/thumbnails/{id}.png?v={version}"),
        None => format!("{VORTEX_BASE}/assets/thumbnails/{id}.png"),
    });

    Some(Game {
        id,
        name,
        description: string_field(&["description"]),
        thumbnail_url,
        creator: string_field(&["creator_name", "creator"]),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_live_api_shape() {
        let body: serde_json::Value = serde_json::from_str(
            r#"{"id":3,"name":"Snowy Peak","description":"it's really chilly up here","creator_id":2,"creator_name":"kostas","thumbnail_version":"afc5ffe7"}"#,
        )
        .unwrap();
        let game = parse_game(&body).unwrap();
        assert_eq!(game.id, 3);
        assert_eq!(game.name, "Snowy Peak");
        assert_eq!(game.creator.as_deref(), Some("kostas"));
        assert_eq!(
            game.thumbnail_url.as_deref(),
            Some("https://playvortex.io/assets/thumbnails/3.png?v=afc5ffe7")
        );
    }

    #[test]
    fn skips_entries_without_name_or_id() {
        assert!(parse_game(&serde_json::json!({"id": 5})).is_none());
        assert!(parse_game(&serde_json::json!({"name": "x"})).is_none());
        assert!(parse_game(&serde_json::json!({"id": 5, "name": ""})).is_none());
    }

    #[test]
    fn thumbnail_without_version_omits_query() {
        let game = parse_game(&serde_json::json!({"id": 7, "name": "n"})).unwrap();
        assert_eq!(
            game.thumbnail_url.as_deref(),
            Some("https://playvortex.io/assets/thumbnails/7.png")
        );
    }
}
