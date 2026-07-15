pub mod auth;
pub mod config;
pub mod crypto;
pub mod doctor;
pub mod error;
pub mod friends;
pub mod games;
pub mod launcher;
pub mod logger;
pub mod net;
pub mod platform;
pub mod playtime;
pub mod plugin;
pub mod presence;
pub mod progress;
pub mod setup;
pub mod shortcuts;
pub mod updater;
pub mod wine_versions;
pub mod uri;

pub use config::Config;
pub use error::RikoError;
pub use progress::{LogLevel, NullSink, ProgressEvent, ProgressSink};

pub const VORTEX_BASE: &str = "https://playvortex.io";
pub const USER_AGENT: &str = "riko-launcher/0.1.0";
