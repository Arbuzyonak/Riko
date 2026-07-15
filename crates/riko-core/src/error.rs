#[derive(Debug, thiserror::Error)]
pub enum RikoError {
    #[error("config error: {0}")]
    Config(String),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("wine error: {0}")]
    Wine(String),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("plugin error: {0}")]
    Plugin(String),
    #[error("setup error: {0}")]
    Setup(String),
    #[error("game {0} is already running")]
    AlreadyRunning(u32),
    #[error("not logged in")]
    NotLoggedIn,
    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for RikoError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
