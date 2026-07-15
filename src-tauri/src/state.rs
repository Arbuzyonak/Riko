use riko_core::launcher::GameHandle;
use riko_core::Config;
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

pub struct AppState {
    pub config: RwLock<Config>,
    pub sessions: Mutex<HashMap<u32, GameHandle>>,
    pub migrated_from_tempest: bool,
}

impl AppState {
    pub fn initialize() -> Self {
        let migrated = Config::migrate_from_tempest();
        let config = Config::load();
        riko_core::logger::init(config.paths.log_file.clone());
        if migrated {
            riko_core::logger::info("migrated configuration from tempest");
        }
        Self {
            config: RwLock::new(config),
            sessions: Mutex::new(HashMap::new()),
            migrated_from_tempest: migrated,
        }
    }
}
