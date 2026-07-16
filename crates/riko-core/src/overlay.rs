use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayState {
    pub game_id: u32,
    pub game_name: String,
    pub started_at_unix: i64,
    #[serde(default)]
    pub friends_online: u32,
}

pub fn state_path() -> PathBuf {
    Config::data_dir().join("overlay-state.json")
}

pub fn write(state: &OverlayState) {
    if let Ok(json) = serde_json::to_string(state) {
        std::fs::create_dir_all(Config::data_dir()).ok();
        let path = state_path();
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            std::fs::rename(&tmp, &path).ok();
        }
    }
}

pub fn read() -> Option<OverlayState> {
    serde_json::from_str(&std::fs::read_to_string(state_path()).ok()?).ok()
}

pub fn clear() {
    std::fs::remove_file(state_path()).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_and_clears() {
        let state = OverlayState {
            game_id: 3,
            game_name: "Snowy Peak".to_string(),
            started_at_unix: 1_700_000_000,
            friends_online: 2,
        };
        write(&state);
        assert_eq!(read().as_ref(), Some(&state));
        clear();
        assert!(read().is_none());
    }
}
