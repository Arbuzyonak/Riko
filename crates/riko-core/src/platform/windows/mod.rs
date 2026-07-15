use crate::config::Config;
use crate::doctor::{CheckResult, FixAction};
use crate::plugin::ResolvedPluginEnv;
use crate::progress::ProgressSink;
use crate::setup::{PlannedStep, SetupPlan, SetupStep, StepStatus};
use crate::RikoError;
use std::path::Path;
use std::process::Command;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

const PROTOCOL_KEY: &str = r"Software\Classes\vortex";

pub fn register_uri() -> Result<(), RikoError> {
    let exe = std::env::current_exe()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(PROTOCOL_KEY)?;
    key.set_value("", &"URL:Vortex Protocol")?;
    key.set_value("URL Protocol", &"")?;
    let (cmd, _) = hkcu.create_subkey(format!(r"{PROTOCOL_KEY}\shell\open\command"))?;
    cmd.set_value("", &format!("\"{}\" \"%1\"", exe.display()))?;
    Ok(())
}

pub fn unregister_uri() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.delete_subkey_all(PROTOCOL_KEY).ok();
}

fn registered_uri_command() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(format!(r"{PROTOCOL_KEY}\shell\open\command"))
        .ok()?;
    key.get_value::<String, _>("").ok()
}

pub fn uri_handler_registered() -> bool {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    match (registered_uri_command(), exe) {
        (Some(cmd), Some(exe)) => cmd.to_lowercase().contains(&exe),
        _ => false,
    }
}

pub fn build_launch_command(cfg: &Config, uri: &str, plugin_env: &ResolvedPluginEnv) -> Command {
    let mut cmd = Command::new(&cfg.paths.vortex_exe);
    for (key, value) in &plugin_env.env {
        cmd.env(key, value);
    }
    if !plugin_env.vulkan_layer_dirs.is_empty() {
        let joined = plugin_env
            .vulkan_layer_dirs
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(";");
        cmd.env("VK_ADD_IMPLICIT_LAYER_PATH", joined);
    }
    cmd.arg(uri);
    cmd
}

pub fn build_receiver_command(_cfg: &Config, path: &Path) -> Command {
    Command::new(path)
}

pub fn receiver_running() -> bool {
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq receiver.exe", "/NH"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("receiver.exe"))
        .unwrap_or(false)
}

pub fn setup_plan(cfg: &Config) -> SetupPlan {
    let status = |done: bool| if done { StepStatus::Done } else { StepStatus::Needed };
    SetupPlan {
        steps: vec![
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
                description: "Let the browser launch games through Riko".to_string(),
                status: status(uri_handler_registered()),
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
    match step {
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
            "step {step:?} is not applicable on Windows"
        ))),
    }
}

pub fn doctor_checks(cfg: &Config) -> Vec<CheckResult> {
    let mut checks = vec![];

    if uri_handler_registered() {
        checks.push(CheckResult::pass("uri-handler", "URI handler", "registered"));
    } else {
        checks.push(CheckResult::fail(
            "uri-handler",
            "URI handler",
            "not registered",
            FixAction::register_uri(),
        ));
    }

    match std::process::Command::new("vulkaninfo").arg("--summary").output() {
        Ok(out) if out.status.success() => {
            checks.push(CheckResult::pass("vulkan", "Vulkan", "loader responding"));
        }
        _ => checks.push(CheckResult::pass(
            "vulkan",
            "Vulkan",
            "vulkaninfo not available (usually fine on Windows)",
        )),
    }

    let _ = cfg;
    checks
}

pub fn uninstall(cfg: &Config) -> Result<(), RikoError> {
    unregister_uri();
    let data_dir = Config::data_dir();
    let config_dir = Config::config_dir();
    for path in [&data_dir, &config_dir] {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
    }
    let _ = cfg;
    Ok(())
}
