use crate::config::Config;
use crate::net;
use serde::Serialize;

pub const DEFAULT_ENDPOINT: &str = "https://riko-telemetry.workers.dev";

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub enabled: bool,
    pub endpoint: String,
    pub install_id: Option<String>,
    pub version: &'static str,
}

impl Snapshot {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            enabled: cfg.telemetry.enabled,
            endpoint: endpoint(cfg),
            install_id: cfg.telemetry.install_id.clone(),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

pub fn endpoint(cfg: &Config) -> String {
    cfg.telemetry
        .endpoint
        .clone()
        .filter(|e| !e.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string())
}

pub fn ensure_install_id(cfg: &mut Config) -> String {
    if let Some(id) = &cfg.telemetry.install_id
        && !id.is_empty()
    {
        return id.clone();
    }
    let id = random_id();
    cfg.telemetry.install_id = Some(id.clone());
    id
}

fn random_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    let mut out = String::with_capacity(32);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "other"
    }
}

#[derive(Serialize)]
struct Heartbeat<'a> {
    install_id: &'a str,
    version: &'a str,
    os: &'a str,
    arch: &'a str,
}

pub async fn heartbeat(snapshot: &Snapshot) {
    if !snapshot.enabled {
        return;
    }
    let Some(id) = snapshot.install_id.as_deref() else {
        return;
    };
    let body = Heartbeat {
        install_id: id,
        version: snapshot.version,
        os: os_name(),
        arch: std::env::consts::ARCH,
    };
    let _ = net::shared()
        .post(format!("{}/heartbeat", snapshot.endpoint))
        .json(&body)
        .send()
        .await;
}

#[derive(Serialize)]
struct ErrorReport<'a> {
    install_id: Option<&'a str>,
    version: &'a str,
    os: &'a str,
    kind: &'a str,
    message: String,
}

pub async fn report_error(snapshot: &Snapshot, kind: &str, message: &str) {
    if !snapshot.enabled {
        return;
    }
    let body = ErrorReport {
        install_id: snapshot.install_id.as_deref(),
        version: snapshot.version,
        os: os_name(),
        kind,
        message: message.chars().take(2000).collect(),
    };
    let _ = net::shared()
        .post(format!("{}/error", snapshot.endpoint))
        .json(&body)
        .send()
        .await;
}

pub fn install_panic_hook(snapshot: Snapshot) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info.to_string();
        tracing::error!("panic: {message}");
        if snapshot.enabled
            && let Ok(handle) = tokio::runtime::Handle::try_current()
        {
            let snapshot = snapshot.clone();
            handle.spawn(async move {
                report_error(&snapshot, "panic", &message).await;
            });
        }
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_id_is_stable_and_hex() {
        let mut cfg = Config::default();
        let first = ensure_install_id(&mut cfg);
        assert_eq!(first.len(), 32);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(ensure_install_id(&mut cfg), first);
    }

    #[test]
    fn endpoint_falls_back_to_default() {
        let mut cfg = Config::default();
        assert_eq!(endpoint(&cfg), DEFAULT_ENDPOINT);
        cfg.telemetry.endpoint = Some("https://t.example.com".to_string());
        assert_eq!(endpoint(&cfg), "https://t.example.com");
    }

    #[test]
    fn disabled_snapshot_sends_nothing() {
        let cfg = Config::default();
        let snap = Snapshot::from_config(&cfg);
        assert!(!snap.enabled);
    }
}
