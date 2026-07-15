pub mod process;

use crate::config::Config;
use crate::plugin::ResolvedPluginEnv;
use crate::RikoError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};

const NOISE_PATTERNS: &[&str] = &[
    "fixme:",
    "libEGL warning",
    "pci id for fd",
    "wine-staging",
    "experimental patches",
    "DxgiFactory::QueryInterface",
    "DxgiAdapter::QueryInterface",
    "create_factory_media",
    "EnableNonClientDpiScaling",
    "DwmSetWindowAttribute",
];

fn is_noise(line: &str) -> bool {
    NOISE_PATTERNS.iter().any(|p| line.contains(p))
}

#[derive(Clone, Debug, Serialize)]
pub struct GameSession {
    pub game_id: u32,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameEvent {
    Started { pid: u32 },
    Log { line: String, is_stderr: bool },
    Exited { code: Option<i32>, duration_secs: u64 },
}

pub struct GameHandle {
    pub session: GameSession,
    kill: oneshot::Sender<()>,
}

impl GameHandle {
    pub fn terminate(self) {
        let _ = self.kill.send(());
    }
}

pub async fn launch(
    cfg: &Config,
    game_id: u32,
    uri: String,
    plugin_env: ResolvedPluginEnv,
    events: mpsc::UnboundedSender<GameEvent>,
) -> Result<GameHandle, RikoError> {
    if !cfg.paths.vortex_exe.exists() {
        return Err(RikoError::Setup(format!(
            "Vortex.exe not found at {}; run setup first",
            cfg.paths.vortex_exe.display()
        )));
    }

    let mut receiver = process::ProcessManager::new();
    receiver.ensure_receiver(cfg);

    let sidecars = plugin_env.sidecars.clone();
    let mut cmd =
        tokio::process::Command::from(crate::platform::build_launch_command(cfg, &uri, &plugin_env));
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        RikoError::Wine(format!(
            "failed to launch (is {} installed?): {e}",
            cfg.wine.binary
        ))
    })?;

    let pid = child.id().unwrap_or_default();
    let started_at = Utc::now();
    let session = GameSession {
        game_id,
        pid,
        started_at,
    };
    let (kill_tx, mut kill_rx) = oneshot::channel::<()>();

    events.send(GameEvent::Started { pid }).ok();
    crate::playtime::record_launch(game_id);

    let filter = cfg.launcher.filter_wine_noise;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(out) = stdout {
        let tx = events.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if filter && is_noise(&line) {
                    continue;
                }
                tx.send(GameEvent::Log {
                    line,
                    is_stderr: false,
                })
                .ok();
            }
        });
    }

    if let Some(err) = stderr {
        let tx = events.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if filter && is_noise(&line) {
                    continue;
                }
                tx.send(GameEvent::Log {
                    line,
                    is_stderr: true,
                })
                .ok();
            }
        });
    }

    for sidecar in sidecars {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(sidecar.delay_secs)).await;
            match tokio::process::Command::new(&sidecar.path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(mut child) => {
                    child.wait().await.ok();
                }
                Err(e) => tracing::warn!("sidecar {} failed: {}", sidecar.path.display(), e),
            }
        });
    }

    tokio::spawn(async move {
        let status = tokio::select! {
            status = child.wait() => status,
            _ = &mut kill_rx => {
                terminate_child(&mut child, pid).await;
                child.wait().await
            }
        };
        drop(receiver);
        let duration_secs = (Utc::now() - started_at).num_seconds().max(0) as u64;
        crate::playtime::add_seconds(game_id, duration_secs);
        events
            .send(GameEvent::Exited {
                code: status.ok().and_then(|s| s.code()),
                duration_secs,
            })
            .ok();
    });

    Ok(GameHandle {
        session,
        kill: kill_tx,
    })
}

#[cfg(unix)]
async fn terminate_child(child: &mut tokio::process::Child, pid: u32) {
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
    let graceful =
        tokio::time::timeout(std::time::Duration::from_secs(10), child.wait()).await;
    if graceful.is_err() {
        child.kill().await.ok();
    }
}

#[cfg(windows)]
async fn terminate_child(child: &mut tokio::process::Child, _pid: u32) {
    child.kill().await.ok();
}
