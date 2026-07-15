use crate::config::Config;
use serde::Serialize;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Clone, Debug, Serialize)]
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub detail: String,
    pub fix: Option<FixAction>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FixAction {
    pub label: String,
    #[serde(flatten)]
    pub kind: FixKind,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FixKind {
    Command { shell: String },
    RunSetup,
    RunLogin,
    RunUpdate,
    RegisterUri,
}

impl CheckResult {
    pub fn pass(id: &str, name: &str, detail: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            passed: true,
            detail: detail.into(),
            fix: None,
        }
    }

    pub fn fail(id: &str, name: &str, detail: impl Into<String>, fix: Option<FixAction>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            passed: false,
            detail: detail.into(),
            fix,
        }
    }
}

impl FixAction {
    pub fn command(label: &str, shell: impl Into<String>) -> Option<Self> {
        Some(Self {
            label: label.to_string(),
            kind: FixKind::Command {
                shell: shell.into(),
            },
        })
    }

    pub fn setup() -> Option<Self> {
        Some(Self {
            label: "Run setup".to_string(),
            kind: FixKind::RunSetup,
        })
    }

    pub fn login() -> Option<Self> {
        Some(Self {
            label: "Log in".to_string(),
            kind: FixKind::RunLogin,
        })
    }

    pub fn update() -> Option<Self> {
        Some(Self {
            label: "Download Vortex".to_string(),
            kind: FixKind::RunUpdate,
        })
    }

    pub fn register_uri() -> Option<Self> {
        Some(Self {
            label: "Register URI handler".to_string(),
            kind: FixKind::RegisterUri,
        })
    }
}

pub fn run_checks(cfg: &Config) -> Vec<CheckResult> {
    let mut checks = crate::platform::doctor_checks(cfg);
    checks.push(check_vortex_exe(cfg));
    checks.push(check_session(cfg));
    checks.push(check_network());
    checks
}

fn check_vortex_exe(cfg: &Config) -> CheckResult {
    if cfg.paths.vortex_exe.exists() {
        let size = std::fs::metadata(&cfg.paths.vortex_exe)
            .map(|m| format!("{:.1} MB", m.len() as f64 / 1_000_000.0))
            .unwrap_or_default();
        CheckResult::pass("vortex-exe", "Vortex client", size)
    } else {
        CheckResult::fail(
            "vortex-exe",
            "Vortex client",
            "Vortex.exe not found",
            FixAction::update(),
        )
    }
}

fn check_session(cfg: &Config) -> CheckResult {
    if cfg.auth.session_token.is_some() {
        CheckResult::pass("session", "Session token", "stored")
    } else {
        CheckResult::fail("session", "Session token", "not logged in", FixAction::login())
    }
}

fn check_network() -> CheckResult {
    let reachable = std::net::ToSocketAddrs::to_socket_addrs(&("playvortex.io", 443u16))
        .ok()
        .and_then(|mut addrs| addrs.next())
        .map(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(5)).is_ok())
        .unwrap_or(false);
    if reachable {
        CheckResult::pass("network", "Network", "playvortex.io:443 reachable")
    } else {
        CheckResult::fail(
            "network",
            "Network",
            "playvortex.io:443 unreachable; check your connection or firewall",
            None,
        )
    }
}
