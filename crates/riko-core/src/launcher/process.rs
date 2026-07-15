use crate::config::Config;
use std::path::PathBuf;
use std::process::{Child, Stdio};

pub struct ProcessManager {
    receiver: Option<Child>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self { receiver: None }
    }

    pub fn receiver_path(cfg: &Config) -> PathBuf {
        cfg.paths
            .vortex_exe
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("receiver.exe")
    }

    pub fn ensure_receiver(&mut self, cfg: &Config) {
        let path = Self::receiver_path(cfg);

        if !path.exists() {
            tracing::debug!("receiver.exe not found at {}", path.display());
            return;
        }

        if crate::platform::receiver_running() {
            tracing::debug!("receiver.exe already running");
            return;
        }

        let mut cmd = crate::platform::build_receiver_command(cfg, &path);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        match cmd.spawn() {
            Ok(child) => {
                tracing::debug!("receiver.exe started (pid {})", child.id());
                self.receiver = Some(child);
            }
            Err(e) => tracing::warn!("could not start receiver.exe: {}", e),
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(mut child) = self.receiver.take() {
            child.kill().ok();
            child.wait().ok();
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
