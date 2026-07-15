use crate::RikoError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub accounts: Vec<StoredAccount>,
    #[serde(default)]
    pub paths: PathConfig,
    #[serde(default)]
    pub wine: WineConfig,
    #[serde(default)]
    pub launcher: LauncherConfig,
    #[serde(default)]
    pub plugins: PluginConfig,
    #[serde(default)]
    pub presence: PresenceConfig,
    #[serde(default)]
    pub launch_overrides: HashMap<String, LaunchOverrides>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LaunchOverrides {
    #[serde(default)]
    pub wine_binary: Option<String>,
    #[serde(default)]
    pub use_esync: Option<bool>,
    #[serde(default)]
    pub use_fsync: Option<bool>,
    #[serde(default)]
    pub use_gamemode: Option<bool>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl LaunchOverrides {
    pub fn is_empty(&self) -> bool {
        self.wine_binary.as_deref().is_none_or(str::is_empty)
            && self.use_esync.is_none()
            && self.use_fsync.is_none()
            && self.use_gamemode.is_none()
            && self.env.is_empty()
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AuthConfig {
    pub session_token: Option<String>,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredAccount {
    pub username: String,
    pub session_token: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PathConfig {
    pub wine_prefix: PathBuf,
    pub vortex_exe: PathBuf,
    pub log_file: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WineConfig {
    pub binary: String,
    pub env: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LauncherConfig {
    pub filter_wine_noise: bool,
    pub auto_update: bool,
    pub use_esync: bool,
    pub use_fsync: bool,
    pub use_gamemode: bool,
    pub shader_cache: bool,
    #[serde(default)]
    pub minimize_while_playing: bool,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PluginConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub per_game: HashMap<String, PerGamePlugins>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PerGamePlugins {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub disabled: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PresenceConfig {
    pub enabled: bool,
}

impl Default for PathConfig {
    fn default() -> Self {
        let data = Config::data_dir();
        Self {
            wine_prefix: data.join("prefix"),
            vortex_exe: data.join("Vortex.exe"),
            log_file: data.join("riko.log"),
        }
    }
}

impl Default for WineConfig {
    fn default() -> Self {
        Self {
            binary: "wine".to_string(),
            env: HashMap::new(),
        }
    }
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            filter_wine_noise: true,
            auto_update: true,
            use_esync: true,
            use_fsync: true,
            use_gamemode: false,
            shader_cache: true,
            minimize_while_playing: false,
        }
    }
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        std::env::var_os("RIKO_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("~/.config"))
                    .join("riko")
            })
    }

    pub fn data_dir() -> PathBuf {
        std::env::var_os("RIKO_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("~/.local/share"))
                    .join("riko")
            })
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn migrate_from_tempest() -> bool {
        if Self::config_file().exists() {
            return false;
        }
        let tempest_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("tempest");
        let src_config = tempest_dir.join("config.toml");
        if !src_config.exists() {
            return false;
        }
        if std::fs::create_dir_all(Self::config_dir()).is_err() {
            return false;
        }
        let copied = std::fs::copy(&src_config, Self::config_file()).is_ok();
        let src_key = tempest_dir.join("vortex.key");
        if src_key.exists() {
            std::fs::copy(&src_key, Self::config_dir().join("vortex.key")).ok();
        }
        if copied {
            let mut cfg = Self::load();
            cfg.paths.log_file = Self::data_dir().join("riko.log");
            cfg.plugins.enabled = migrate_tempest_plugins();
            cfg.save().ok();
        }
        copied
    }

    pub fn effective_for_game(&self, game_id: u32) -> Self {
        let mut cfg = self.clone();
        if let Some(overrides) = self.launch_overrides.get(&game_id.to_string()) {
            if let Some(binary) = overrides.wine_binary.clone().filter(|b| !b.is_empty()) {
                cfg.wine.binary = binary;
            }
            if let Some(v) = overrides.use_esync {
                cfg.launcher.use_esync = v;
            }
            if let Some(v) = overrides.use_fsync {
                cfg.launcher.use_fsync = v;
            }
            if let Some(v) = overrides.use_gamemode {
                cfg.launcher.use_gamemode = v;
            }
            cfg.wine.env.extend(overrides.env.clone());
        }
        cfg
    }

    pub fn load() -> Self {
        let path = Self::config_file();
        if !path.exists() {
            return Self::default();
        }
        let contents = std::fs::read_to_string(&path).unwrap_or_default();

        #[derive(serde::Deserialize)]
        struct RawConfig {
            auth: RawAuth,
        }

        #[derive(serde::Deserialize)]
        struct RawAuth {
            session_token: Option<String>,
        }

        let raw: RawConfig = toml::from_str(&contents).unwrap_or(RawConfig {
            auth: RawAuth {
                session_token: None,
            },
        });

        let mut cfg: Self = toml::from_str(&contents).unwrap_or_else(|e| {
            tracing::warn!("config parse failed, using defaults: {}", e);
            Self::default()
        });

        if let Some(ref token) = raw.auth.session_token
            && let Some(decrypted) = crate::crypto::decrypt(token)
        {
            cfg.auth.session_token = Some(decrypted);
        }

        for account in &mut cfg.accounts {
            if let Some(decrypted) = crate::crypto::decrypt(&account.session_token) {
                account.session_token = decrypted;
            }
        }

        if let (Some(username), Some(token)) = (&cfg.auth.username, &cfg.auth.session_token)
            && !cfg
                .accounts
                .iter()
                .any(|a| a.username.eq_ignore_ascii_case(username))
        {
            cfg.accounts.push(StoredAccount {
                username: username.clone(),
                session_token: token.clone(),
            });
        }

        cfg
    }

    pub fn save(&self) -> Result<(), RikoError> {
        let mut cloned = self.clone();

        if let Some(ref token) = cloned.auth.session_token.clone()
            && crate::crypto::decrypt(token).is_none()
        {
            cloned.auth.session_token = Some(crate::crypto::encrypt(token));
        }

        for account in &mut cloned.accounts {
            if crate::crypto::decrypt(&account.session_token).is_none() {
                account.session_token = crate::crypto::encrypt(&account.session_token);
            }
        }

        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let contents =
            toml::to_string_pretty(&cloned).map_err(|e| RikoError::Config(e.to_string()))?;
        std::fs::write(dir.join("config.toml"), contents)?;
        Ok(())
    }
}

fn migrate_tempest_plugins() -> Vec<String> {
    let src = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("tempest")
        .join("plugins");
    if !src.is_dir() {
        return vec![];
    }
    let dest = crate::plugin::plugins_dir();
    let mut names = vec![];
    if let Ok(entries) = std::fs::read_dir(&src) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            let target = dest.join(&name);
            if !target.exists()
                && crate::plugin::copy_dir_recursive(&entry.path(), &target).is_ok()
            {
                names.push(name);
            }
        }
    }
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let cfg = Config::default();
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let loaded: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(
            loaded.launcher.filter_wine_noise,
            cfg.launcher.filter_wine_noise
        );
        assert_eq!(loaded.launcher.auto_update, cfg.launcher.auto_update);
        assert_eq!(loaded.wine.binary, cfg.wine.binary);
        assert_eq!(loaded.presence.enabled, cfg.presence.enabled);
    }

    #[test]
    fn parses_tempest_config() {
        let tempest_toml = r#"
[auth]
username = "player"

[paths]
wine_prefix = "/home/user/.local/share/tempest/prefix"
vortex_exe = "/home/user/.local/share/tempest/Vortex.exe"
log_file = "/home/user/.local/share/tempest/tempest.log"

[wine]
binary = "wine"

[wine.env]

[launcher]
filter_wine_noise = true
auto_update = true
use_esync = true
use_fsync = false
use_gamemode = false
shader_cache = true
"#;
        let cfg: Config = toml::from_str(tempest_toml).unwrap();
        assert_eq!(cfg.auth.username.as_deref(), Some("player"));
        assert!(!cfg.launcher.use_fsync);
        assert!(cfg.plugins.enabled.is_empty());
        assert!(cfg.presence.enabled);
        assert_eq!(
            cfg.paths.wine_prefix,
            PathBuf::from("/home/user/.local/share/tempest/prefix")
        );
    }
}
