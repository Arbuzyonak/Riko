use super::manifest::PluginManifest;
use crate::progress::ProgressSink;
use crate::RikoError;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

const STAGE: &str = "plugin";
const BUILD_TIMEOUT_SECS: u64 = 120;

pub async fn build(
    dir: &Path,
    manifest: &PluginManifest,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    let Some(spec) = &manifest.build else {
        return verify_artifacts(dir, manifest);
    };

    sink.started(STAGE, &format!("Building {}", manifest.plugin.name));
    sink.info(STAGE, &format!("$ {}", spec.command));

    let mut cmd = shell_command(dir, &spec.command);
    cmd.current_dir(dir);
    cmd.env_clear();
    for key in ["PATH", "HOME", "CC", "VULKAN_INCLUDE"] {
        if let Ok(value) = std::env::var(key) {
            cmd.env(key, value);
        }
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        RikoError::Plugin(format!("failed to start build command: {e}"))
    })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let out_task = async {
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                sink.info(STAGE, &line);
            }
        }
    };
    let err_task = async {
        if let Some(err) = stderr {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                sink.warn(STAGE, &line);
            }
        }
    };

    let waited = tokio::time::timeout(
        std::time::Duration::from_secs(BUILD_TIMEOUT_SECS),
        async {
            tokio::join!(out_task, err_task);
            child.wait().await
        },
    )
    .await;

    let status = match waited {
        Ok(status) => status?,
        Err(_) => {
            child.kill().await.ok();
            sink.finished(STAGE, false, Some("build timed out".to_string()));
            return Err(RikoError::Plugin(format!(
                "build timed out after {BUILD_TIMEOUT_SECS}s"
            )));
        }
    };
    if !status.success() {
        sink.finished(STAGE, false, Some(format!("build failed with {status}")));
        return Err(RikoError::Plugin(format!("build failed with {status}")));
    }

    verify_artifacts(dir, manifest)?;
    sink.finished(STAGE, true, None);
    Ok(())
}

fn shell_command(dir: &Path, command: &str) -> tokio::process::Command {
    #[cfg(unix)]
    {
        if which::which("bwrap").is_ok() {
            let mut cmd = tokio::process::Command::new("bwrap");
            cmd.args(["--ro-bind", "/", "/"]);
            cmd.arg("--bind").arg(dir).arg(dir);
            cmd.args(["--dev", "/dev", "--proc", "/proc", "--unshare-net", "--die-with-parent"]);
            cmd.args(["sh", "-c", command]);
            return cmd;
        }
        let mut cmd = tokio::process::Command::new("sh");
        cmd.args(["-c", command]);
        cmd
    }
    #[cfg(windows)]
    {
        let _ = dir;
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.args(["/C", command]);
        cmd
    }
}

pub fn verify_artifacts(dir: &Path, manifest: &PluginManifest) -> Result<(), RikoError> {
    if let Some(spec) = &manifest.build {
        for artifact in &spec.artifacts {
            if !dir.join(artifact).exists() {
                return Err(RikoError::Plugin(format!(
                    "expected build artifact '{artifact}' is missing"
                )));
            }
        }
    }
    Ok(())
}

pub fn artifacts_present(dir: &Path, manifest: &PluginManifest) -> bool {
    verify_artifacts(dir, manifest).is_ok()
}
