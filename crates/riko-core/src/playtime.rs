use crate::config::Config;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlaytimeEntry {
    pub total_secs: u64,
    pub last_played: Option<DateTime<Utc>>,
    pub launches: u32,
}

fn store_path() -> PathBuf {
    Config::data_dir().join("playtime.json")
}

pub fn load() -> HashMap<u32, PlaytimeEntry> {
    std::fs::read_to_string(store_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(entries: &HashMap<u32, PlaytimeEntry>) {
    let path = store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(json) = serde_json::to_string_pretty(entries) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            std::fs::rename(&tmp, &path).ok();
        }
    }
}

pub fn record_launch(game_id: u32) {
    let mut entries = load();
    let entry = entries.entry(game_id).or_default();
    entry.launches += 1;
    entry.last_played = Some(Utc::now());
    save(&entries);
}

pub fn add_seconds(game_id: u32, secs: u64) {
    if secs == 0 {
        return;
    }
    let mut entries = load();
    let entry = entries.entry(game_id).or_default();
    entry.total_secs += secs;
    entry.last_played = Some(Utc::now());
    save(&entries);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub game_id: u32,
    pub started_at: DateTime<Utc>,
    pub duration_secs: u64,
}

const SESSION_LIMIT: usize = 500;

fn sessions_path() -> PathBuf {
    Config::data_dir().join("sessions.json")
}

pub fn load_sessions() -> Vec<SessionRecord> {
    std::fs::read_to_string(sessions_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn record_session(game_id: u32, started_at: DateTime<Utc>, duration_secs: u64) {
    let mut sessions = load_sessions();
    sessions.push(SessionRecord {
        game_id,
        started_at,
        duration_secs,
    });
    if sessions.len() > SESSION_LIMIT {
        let excess = sessions.len() - SESSION_LIMIT;
        sessions.drain(..excess);
    }
    if let Ok(json) = serde_json::to_string(&sessions) {
        let path = sessions_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            std::fs::rename(&tmp, &path).ok();
        }
    }
}
