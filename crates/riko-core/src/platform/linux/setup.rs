use crate::config::Config;
use crate::progress::ProgressSink;
use crate::setup::{PlannedStep, SetupPlan, SetupStep, StepStatus};
use crate::RikoError;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

pub enum Distro {
    Fedora,
    Debian,
    Arch,
    OpenSuse,
    Unknown(String),
}

impl std::fmt::Display for Distro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Distro::Fedora => write!(f, "Fedora/RHEL"),
            Distro::Debian => write!(f, "Debian/Ubuntu"),
            Distro::Arch => write!(f, "Arch Linux"),
            Distro::OpenSuse => write!(f, "openSUSE"),
            Distro::Unknown(s) => write!(f, "Unknown ({s})"),
        }
    }
}

pub fn detect_distro() -> Distro {
    let contents = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let mut id = String::new();
    let mut id_like = String::new();
    for line in contents.lines() {
        if let Some(val) = line.strip_prefix("ID=") {
            id = val.trim_matches('"').to_lowercase();
        } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
            id_like = val.trim_matches('"').to_lowercase();
        }
    }
    let check = |s: &str| id == s || id_like.contains(s);
    if check("fedora") || check("rhel") || check("centos") {
        Distro::Fedora
    } else if check("debian") || check("ubuntu") {
        Distro::Debian
    } else if check("arch") {
        Distro::Arch
    } else if check("opensuse") || check("suse") {
        Distro::OpenSuse
    } else {
        Distro::Unknown(id)
    }
}

struct DistroCommands {
    update: &'static str,
    wine_packages: &'static str,
    extra_setup: Option<&'static str>,
    winetricks_manual: bool,
    gamemode: Option<&'static str>,
}

fn distro_commands(distro: &Distro) -> DistroCommands {
    match distro {
        Distro::Fedora => DistroCommands {
            update: "dnf upgrade --refresh -y",
            wine_packages: "dnf install -y wine winetricks wine.i686",
            extra_setup: None,
            winetricks_manual: false,
            gamemode: Some("dnf install -y gamemode"),
        },
        Distro::Debian => DistroCommands {
            update: "apt update && apt upgrade -y",
            wine_packages: "apt install -y wine64 wine32",
            extra_setup: Some("dpkg --add-architecture i386 && apt update"),
            winetricks_manual: true,
            gamemode: Some("apt install -y gamemode"),
        },
        Distro::Arch => DistroCommands {
            update: "pacman -Syu --noconfirm",
            wine_packages: "pacman -S --noconfirm --needed wine winetricks",
            extra_setup: None,
            winetricks_manual: false,
            gamemode: Some("pacman -S --noconfirm --needed gamemode"),
        },
        Distro::OpenSuse => DistroCommands {
            update: "zypper refresh && zypper update -y",
            wine_packages: "zypper install -y wine winetricks wine-32bit",
            extra_setup: None,
            winetricks_manual: false,
            gamemode: Some("zypper install -y gamemode"),
        },
        Distro::Unknown(_) => DistroCommands {
            update: "",
            wine_packages: "",
            extra_setup: None,
            winetricks_manual: false,
            gamemode: None,
        },
    }
}

const WINETRICKS_INSTALL: &str = "curl -L https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks -o /usr/local/bin/winetricks && chmod +x /usr/local/bin/winetricks";

fn wine_install_script(cmds: &DistroCommands) -> String {
    let mut parts = Vec::new();
    if let Some(extra) = cmds.extra_setup {
        parts.push(extra.to_string());
    }
    if !cmds.update.is_empty() {
        parts.push(cmds.update.to_string());
    }
    if !cmds.wine_packages.is_empty() {
        parts.push(cmds.wine_packages.to_string());
    }
    if cmds.winetricks_manual {
        parts.push(WINETRICKS_INSTALL.to_string());
    }
    parts.join(" && ")
}

pub fn setup_plan(cfg: &Config) -> SetupPlan {
    let distro = detect_distro();
    let cmds = distro_commands(&distro);
    let prefix = &cfg.paths.wine_prefix;

    let wine_ok = which::which("wine").is_ok();
    let prefix_ok = prefix.join("system.reg").exists();
    let winetricks_done = prefix.join("winetricks.log").exists();
    let gamemode_ok = which::which("gamemoderun").is_ok();
    let dxvk_ok = super::dll::verify_dll(&prefix.join("drive_c/windows/system32/dxgi.dll"));
    let vkd3d_ok = super::dll::verify_dll(&prefix.join("drive_c/windows/system32/d3d12.dll"));
    let vortex_ok = cfg.paths.vortex_exe.exists();
    let uri_ok = uri_handler_registered();

    let status = |done: bool| if done { StepStatus::Done } else { StepStatus::Needed };
    let wine_script = wine_install_script(&cmds);

    SetupPlan {
        steps: vec![
            PlannedStep {
                step: SetupStep::InstallWine,
                label: "Install Wine".to_string(),
                description: format!("Install Wine and winetricks ({distro})"),
                status: status(wine_ok),
                manual_command: (!wine_script.is_empty())
                    .then(|| format!("sudo sh -c '{wine_script}'")),
            },
            PlannedStep {
                step: SetupStep::CreatePrefix,
                label: "Create Wine prefix".to_string(),
                description: format!("Initialise a Wine prefix at {}", prefix.display()),
                status: status(prefix_ok),
                manual_command: None,
            },
            PlannedStep {
                step: SetupStep::InstallWinetricksComponents,
                label: "Install runtime components".to_string(),
                description: "Install d3dcompiler_47, vcrun2022 and corefonts into the prefix"
                    .to_string(),
                status: status(winetricks_done),
                manual_command: None,
            },
            PlannedStep {
                step: SetupStep::InstallGamemode,
                label: "Install GameMode".to_string(),
                description: "Optional: reduces latency by switching the CPU governor while playing"
                    .to_string(),
                status: if gamemode_ok {
                    StepStatus::Done
                } else {
                    StepStatus::Optional
                },
                manual_command: cmds.gamemode.map(|c| format!("sudo sh -c '{c}'")),
            },
            PlannedStep {
                step: SetupStep::InstallDxvk,
                label: "Install DXVK".to_string(),
                description: "D3D9/10/11 to Vulkan translation layer".to_string(),
                status: status(dxvk_ok),
                manual_command: None,
            },
            PlannedStep {
                step: SetupStep::InstallVkd3d,
                label: "Install vkd3d-proton".to_string(),
                description: "D3D12 to Vulkan translation layer".to_string(),
                status: status(vkd3d_ok),
                manual_command: None,
            },
            PlannedStep {
                step: SetupStep::DownloadVortex,
                label: "Download Vortex".to_string(),
                description: "Download the Vortex client".to_string(),
                status: status(vortex_ok),
                manual_command: None,
            },
            PlannedStep {
                step: SetupStep::RegisterUri,
                label: "Register vortex:// handler".to_string(),
                description: "Let the browser launch games through Riko".to_string(),
                status: status(uri_ok),
                manual_command: None,
            },
        ],
    }
}

pub fn uri_handler_registered() -> bool {
    std::process::Command::new("xdg-mime")
        .args(["query", "default", "x-scheme-handler/vortex"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("riko"))
        .unwrap_or(false)
}

pub async fn execute_setup_step(
    step: SetupStep,
    cfg: &Config,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    let distro = detect_distro();
    let cmds = distro_commands(&distro);
    let prefix = cfg.paths.wine_prefix.clone();

    match step {
        SetupStep::InstallWine => {
            let script = wine_install_script(&cmds);
            if script.is_empty() {
                return Err(RikoError::Setup(
                    "unknown distro; install wine and winetricks manually".to_string(),
                ));
            }
            run_root_shell("setup", &script, sink).await
        }
        SetupStep::CreatePrefix => {
            std::fs::create_dir_all(&prefix)?;
            run_shell(
                "setup",
                &format!(
                    "WINEPREFIX=\"{}\" WINEDEBUG=-all wine wineboot --init",
                    prefix.display()
                ),
                sink,
            )
            .await
        }
        SetupStep::InstallWinetricksComponents => {
            if which::which("winetricks").is_err() {
                return Err(RikoError::Setup(
                    "winetricks not found; run the Install Wine step first".to_string(),
                ));
            }
            run_shell(
                "setup",
                &format!(
                    "WINEPREFIX=\"{}\" WINEDEBUG=-all winetricks -q d3dcompiler_47 vcrun2022 corefonts",
                    prefix.display()
                ),
                sink,
            )
            .await
        }
        SetupStep::InstallGamemode => {
            let cmd = cmds.gamemode.ok_or_else(|| {
                RikoError::Setup("unknown distro; install gamemode manually".to_string())
            })?;
            run_root_shell("setup", cmd, sink).await
        }
        SetupStep::InstallDxvk => super::dxvk::install(&prefix, sink).await,
        SetupStep::InstallVkd3d => super::vkd3d::install(&prefix, sink).await,
        SetupStep::DownloadVortex => {
            crate::updater::download_vortex(
                &cfg.paths.vortex_exe,
                cfg.auth.session_token.as_deref(),
                sink,
            )
            .await
        }
        SetupStep::RegisterUri => super::register_uri(),
    }
}

pub async fn run_shell(
    stage: &str,
    command: &str,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    sink.info(stage, &format!("$ {command}"));
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    stream_output(stage, &mut child, sink).await;
    let status = child.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err(RikoError::Setup(format!(
            "command failed with status {status}: {command}"
        )))
    }
}

async fn run_root_shell(
    stage: &str,
    command: &str,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    if super::is_root() {
        return run_shell(stage, command, sink).await;
    }
    if which::which("pkexec").is_err() {
        return Err(RikoError::Setup(format!(
            "administrator privileges required but pkexec is not available; run manually: sudo sh -c '{command}'"
        )));
    }
    sink.info(stage, &format!("$ pkexec sh -c '{command}'"));
    let mut child = tokio::process::Command::new("pkexec")
        .arg("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    stream_output(stage, &mut child, sink).await;
    let status = child.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err(RikoError::Setup(format!(
            "command failed with status {status}; run manually: sudo sh -c '{command}'"
        )))
    }
}

async fn stream_output(stage: &str, child: &mut tokio::process::Child, sink: &dyn ProgressSink) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let out_task = async {
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                sink.info(stage, &line);
            }
        }
    };
    let err_task = async {
        if let Some(err) = stderr {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                sink.warn(stage, &line);
            }
        }
    };
    tokio::join!(out_task, err_task);
}

pub fn uninstall(cfg: &Config) -> Result<(), RikoError> {
    super::unregister_uri();
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
