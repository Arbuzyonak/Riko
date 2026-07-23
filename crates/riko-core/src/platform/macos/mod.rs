use crate::config::Config;
use crate::doctor::{CheckResult, FixAction};
use crate::plugin::ResolvedPluginEnv;
use crate::progress::ProgressSink;
use crate::setup::{PlannedStep, SetupPlan, SetupStep, StepStatus};
use crate::RikoError;
use std::path::Path;
use std::process::{Command, Stdio};

const WINE_INSTALL_HINT: &str = "brew install --cask wine-stable";

pub fn register_uri() -> Result<(), RikoError> {
    Ok(())
}

pub fn unregister_uri() {}

pub fn uri_handler_registered() -> bool {
    true
}

const BREW_BINS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];

fn wine_binary(cfg: &Config) -> String {
    let configured = cfg.wine.binary.as_str();
    let bundled = Path::new(configured).starts_with(crate::wine_versions::wine_dir());
    let candidates: &[&str] = if bundled || configured.is_empty() {
        &["wine", "/opt/homebrew/bin/wine", "/usr/local/bin/wine"]
    } else {
        return configured.to_string();
    };
    candidates
        .iter()
        .find(|c| which::which(c).is_ok())
        .map(|c| c.to_string())
        .unwrap_or_else(|| "wine".to_string())
}

fn wine_path_env() -> String {
    let existing = std::env::var("PATH").unwrap_or_default();
    format!("{}:{existing}", BREW_BINS.join(":"))
}

fn wine_available(cfg: &Config) -> bool {
    which::which(wine_binary(cfg)).is_ok()
        || BREW_BINS
            .iter()
            .any(|d| Path::new(d).join("wine").is_file())
}

pub fn build_launch_command(
    cfg: &Config,
    game_id: u32,
    uri: &str,
    plugin_env: &ResolvedPluginEnv,
) -> Command {
    let mut cmd = Command::new(wine_binary(cfg));
    cmd.env("PATH", wine_path_env());
    cmd.env("WINEPREFIX", &cfg.paths.wine_prefix);
    cmd.env("WGPU_BACKEND", "vulkan");

    if cfg.launcher.use_esync {
        cmd.env("WINEESYNC", "1");
    }
    if cfg.launcher.use_fsync {
        cmd.env("WINEFSYNC", "1");
    }

    if cfg.launcher.shader_cache {
        let cache = crate::shader_cache::dir_for(game_id);
        std::fs::create_dir_all(&cache).ok();
        cmd.env("VKD3D_SHADER_CACHE_PATH", &cache);
        cmd.env("DXVK_STATE_CACHE_PATH", &cache);
    }

    for (key, value) in &cfg.wine.env {
        cmd.env(key, value);
    }

    for (key, value) in &plugin_env.env {
        cmd.env(key, value);
    }

    if !plugin_env.vulkan_layer_dirs.is_empty() {
        let joined = plugin_env
            .vulkan_layer_dirs
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":");
        cmd.env("VK_ADD_IMPLICIT_LAYER_PATH", joined);
    }

    cmd.arg(&cfg.paths.vortex_exe);
    cmd.arg(uri);
    cmd
}

pub fn build_receiver_command(cfg: &Config, path: &Path) -> Command {
    let mut cmd = Command::new(wine_binary(cfg));
    cmd.env("PATH", wine_path_env());
    cmd.env("WINEPREFIX", &cfg.paths.wine_prefix);
    cmd.env("WINEDEBUG", "-all");
    cmd.arg(path);
    cmd
}

pub fn receiver_running() -> bool {
    Command::new("pgrep")
        .args(["-f", "receiver.exe"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub fn game_process_running() -> bool {
    Command::new("pgrep")
        .args(["-f", "Vortex.exe"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub fn kill_game_processes(cfg: &Config) {
    Command::new("wineserver")
        .env("WINEPREFIX", &cfg.paths.wine_prefix)
        .arg("-k")
        .status()
        .ok();
    Command::new("pkill")
        .args(["-f", "Vortex.exe"])
        .status()
        .ok();
}

pub fn setup_plan(cfg: &Config) -> SetupPlan {
    let status = |done: bool| if done { StepStatus::Done } else { StepStatus::Needed };
    SetupPlan {
        steps: vec![
            PlannedStep {
                step: SetupStep::InstallWine,
                label: "Install Wine".to_string(),
                description: "Install Wine so the Vortex client can run".to_string(),
                status: status(wine_available(cfg)),
                manual_command: Some(WINE_INSTALL_HINT.to_string()),
            },
            PlannedStep {
                step: SetupStep::CreatePrefix,
                label: "Create Wine prefix".to_string(),
                description: "Initialize the Wine prefix Vortex runs in".to_string(),
                status: status(cfg.paths.wine_prefix.exists()),
                manual_command: None,
            },
            PlannedStep {
                step: SetupStep::DownloadVortex,
                label: "Download Vortex".to_string(),
                description: "Download the Vortex client".to_string(),
                status: status(cfg.paths.vortex_exe.exists()),
                manual_command: None,
            },
            PlannedStep {
                step: SetupStep::RegisterUri,
                label: "Register vortex:// handler".to_string(),
                description: "Handled by the app bundle on macOS".to_string(),
                status: StepStatus::Done,
                manual_command: None,
            },
        ],
    }
}

pub async fn execute_setup_step(
    step: SetupStep,
    cfg: &Config,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    let prefix = cfg.paths.wine_prefix.clone();
    match step {
        SetupStep::InstallWine => {
            if which::which("brew").is_err() {
                return Err(RikoError::Setup(format!(
                    "Homebrew not found; install Wine manually: {WINE_INSTALL_HINT}"
                )));
            }
            run_shell("setup", WINE_INSTALL_HINT, sink).await
        }
        SetupStep::CreatePrefix => {
            std::fs::create_dir_all(&prefix)?;
            run_shell(
                "setup",
                &format!(
                    "WINEPREFIX=\"{}\" WINEDEBUG=-all {} wineboot --init",
                    prefix.display(),
                    wine_binary(cfg)
                ),
                sink,
            )
            .await
        }
        SetupStep::DownloadVortex => {
            crate::updater::download_vortex(
                &cfg.paths.vortex_exe,
                cfg.auth.session_token.as_deref(),
                sink,
            )
            .await
        }
        SetupStep::RegisterUri => register_uri(),
        _ => Err(RikoError::Setup(format!(
            "step {step:?} is not applicable on macOS"
        ))),
    }
}

async fn run_shell(stage: &str, command: &str, sink: &dyn ProgressSink) -> Result<(), RikoError> {
    use tokio::io::AsyncBufReadExt;
    sink.info(stage, &format!("$ {command}"));
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{command} 2>&1"))
        .env("PATH", wine_path_env())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(out) = child.stdout.take() {
        let mut lines = tokio::io::BufReader::new(out).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            sink.info(stage, &line);
        }
    }
    let status = child.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err(RikoError::Setup(format!(
            "command failed with status {status}: {command}"
        )))
    }
}

pub fn doctor_checks(cfg: &Config) -> Vec<CheckResult> {
    let mut checks = vec![];

    if wine_available(cfg) {
        checks.push(CheckResult::pass("wine", "Wine", "installed"));
    } else {
        checks.push(CheckResult::fail(
            "wine",
            "Wine",
            "not found in PATH",
            FixAction::command("Copy install command", WINE_INSTALL_HINT),
        ));
    }

    if cfg.paths.wine_prefix.exists() {
        checks.push(CheckResult::pass("wine-prefix", "Wine prefix", "initialized"));
    } else {
        checks.push(CheckResult::fail(
            "wine-prefix",
            "Wine prefix",
            "not created",
            FixAction::setup(),
        ));
    }

    checks.push(CheckResult::pass(
        "uri-handler",
        "URI handler",
        "handled by the app bundle",
    ));

    checks
}

pub fn uninstall(cfg: &Config) -> Result<(), RikoError> {
    let data_dir = Config::data_dir();
    let config_dir = Config::config_dir();
    for path in [&data_dir, &config_dir] {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
    }
    if cfg.paths.wine_prefix.exists() && !cfg.paths.wine_prefix.starts_with(&data_dir) {
        std::fs::remove_dir_all(&cfg.paths.wine_prefix)?;
    }
    Ok(())
}
