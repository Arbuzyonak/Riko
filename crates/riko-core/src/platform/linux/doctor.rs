use super::setup::{detect_distro, Distro};
use crate::config::Config;
use crate::doctor::{CheckResult, FixAction};

pub fn doctor_checks(cfg: &Config) -> Vec<CheckResult> {
    let distro = detect_distro();
    let mut checks = vec![];

    match which::which("wine") {
        Ok(path) => {
            let version = wine_version().unwrap_or_else(|| "unknown".to_string());
            checks.push(CheckResult::pass(
                "wine",
                "Wine",
                format!("{} ({version})", path.display()),
            ));
        }
        Err(_) => checks.push(CheckResult::fail(
            "wine",
            "Wine",
            "not found in PATH",
            FixAction::command("Copy install command", wine_install_cmd(&distro)),
        )),
    }

    let prefix_ready = cfg.paths.wine_prefix.join("system.reg").exists();
    if prefix_ready {
        checks.push(CheckResult::pass(
            "prefix",
            "Wine prefix",
            cfg.paths.wine_prefix.display().to_string(),
        ));
    } else {
        let detail = if cfg.paths.wine_prefix.exists() {
            "directory exists but not initialised (wineboot not run)".to_string()
        } else {
            format!("{} does not exist", cfg.paths.wine_prefix.display())
        };
        checks.push(CheckResult::fail("prefix", "Wine prefix", detail, FixAction::setup()));
    }

    match std::process::Command::new("vulkaninfo").arg("--summary").output() {
        Ok(out) if out.status.success() => {
            let output = String::from_utf8_lossy(&out.stdout);
            let gpu = extract_gpu(&output).unwrap_or_else(|| "GPU detected".to_string());
            let gpu_count = output
                .lines()
                .filter(|l| l.trim().starts_with("GPU") && l.trim().ends_with(':'))
                .count();
            checks.push(CheckResult::pass(
                "vulkan",
                "Vulkan",
                format!("{gpu_count} device(s) found"),
            ));
            let lower = gpu.to_lowercase();
            if lower.contains("llvmpipe") || lower.contains("softpipe") {
                checks.push(CheckResult::fail(
                    "gpu",
                    "GPU",
                    format!("{gpu} (software renderer)"),
                    FixAction::command("Copy driver install command", gpu_driver_fix(&distro)),
                ));
            } else {
                checks.push(CheckResult::pass("gpu", "GPU", gpu));
            }
        }
        _ => checks.push(CheckResult::fail(
            "vulkan",
            "Vulkan",
            "vulkaninfo failed",
            FixAction::command("Copy install command", vulkan_fix(&distro)),
        )),
    }

    if is_nvidia() {
        let icd_dir = std::path::Path::new("/usr/share/vulkan/icd.d");
        let found = ["nvidia_icd.json", "nvidia_icd.x86_64.json", "nvidia_icd.i686.json"]
            .iter()
            .any(|name| icd_dir.join(name).exists());
        if found {
            checks.push(CheckResult::pass("nvidia-icd", "NVIDIA Vulkan ICD", "registered"));
        } else {
            checks.push(CheckResult::fail(
                "nvidia-icd",
                "NVIDIA Vulkan ICD",
                "not found",
                FixAction::command("Copy install command", nvidia_fix(&distro)),
            ));
        }
    }

    match std::process::Command::new("xdg-mime")
        .args(["query", "default", "x-scheme-handler/vortex"])
        .output()
    {
        Ok(out) => {
            let handler = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if handler.contains("riko") {
                checks.push(CheckResult::pass("uri-handler", "URI handler", handler));
            } else if handler.is_empty() {
                checks.push(CheckResult::fail(
                    "uri-handler",
                    "URI handler",
                    "not registered",
                    FixAction::register_uri(),
                ));
            } else {
                checks.push(CheckResult::fail(
                    "uri-handler",
                    "URI handler",
                    format!("wrong handler: {handler}"),
                    FixAction::register_uri(),
                ));
            }
        }
        Err(_) => checks.push(CheckResult::fail(
            "uri-handler",
            "URI handler",
            "xdg-mime not found",
            FixAction::command("Copy install command", "Install xdg-utils via your package manager"),
        )),
    }

    match which::which("winetricks") {
        Ok(path) => checks.push(CheckResult::pass(
            "winetricks",
            "Winetricks",
            path.display().to_string(),
        )),
        Err(_) => checks.push(CheckResult::fail(
            "winetricks",
            "Winetricks",
            "not found",
            FixAction::command("Copy install command", winetricks_fix(&distro)),
        )),
    }

    match which::which("gamemoderun") {
        Ok(_) => {
            let detail = if cfg.launcher.use_gamemode {
                "enabled"
            } else {
                "installed (enable it in Settings)"
            };
            checks.push(CheckResult::pass("gamemode", "GameMode", detail));
        }
        Err(_) => checks.push(CheckResult::fail(
            "gamemode",
            "GameMode",
            "not installed (optional but recommended)",
            FixAction::command("Copy install command", gamemode_fix(&distro)),
        )),
    }

    let dxgi = cfg.paths.wine_prefix.join("drive_c/windows/system32/dxgi.dll");
    if super::dll::verify_dll(&dxgi) {
        checks.push(CheckResult::pass("dxvk", "DXVK", "dxgi.dll is a valid PE"));
    } else {
        checks.push(CheckResult::fail(
            "dxvk",
            "DXVK",
            "dxgi.dll not found or invalid",
            FixAction::setup(),
        ));
    }

    let d3d12 = cfg.paths.wine_prefix.join("drive_c/windows/system32/d3d12.dll");
    if super::dll::verify_dll(&d3d12) {
        checks.push(CheckResult::pass("vkd3d", "vkd3d-proton", "d3d12.dll is a valid PE"));
    } else {
        checks.push(CheckResult::fail(
            "vkd3d",
            "vkd3d-proton",
            "d3d12.dll not found or invalid",
            FixAction::setup(),
        ));
    }

    checks
}

fn wine_version() -> Option<String> {
    let out = std::process::Command::new("wine")
        .arg("--version")
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn extract_gpu(vulkaninfo: &str) -> Option<String> {
    let mut in_gpu_block = false;
    let mut current_type = String::new();
    let mut current_name = String::new();

    let usable = |name: &str, ty: &str| {
        !name.is_empty()
            && !ty.contains("CPU")
            && !name.to_lowercase().contains("llvmpipe")
            && !name.to_lowercase().contains("softpipe")
    };

    for line in vulkaninfo.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("GPU") && trimmed.ends_with(':') {
            if in_gpu_block && usable(&current_name, &current_type) {
                return Some(current_name);
            }
            in_gpu_block = true;
            current_type.clear();
            current_name.clear();
        } else if in_gpu_block
            && let Some(eq) = trimmed.find('=')
        {
            let key = trimmed[..eq].trim();
            let val = trimmed[eq + 1..].trim().to_string();
            if key == "deviceName" {
                current_name = val;
            } else if key == "deviceType" {
                current_type = val;
            }
        }
    }

    if in_gpu_block && usable(&current_name, &current_type) {
        return Some(current_name);
    }

    for line in vulkaninfo.lines() {
        let trimmed = line.trim();
        if let Some(eq) = trimmed.find('=')
            && trimmed[..eq].trim() == "deviceName"
        {
            return Some(trimmed[eq + 1..].trim().to_string());
        }
    }
    None
}

fn is_nvidia() -> bool {
    std::path::Path::new("/dev/nvidia0").exists()
        || std::path::Path::new("/proc/driver/nvidia").exists()
}

fn wine_install_cmd(distro: &Distro) -> String {
    match distro {
        Distro::Fedora => "sudo dnf install wine winetricks wine.i686".to_string(),
        Distro::Debian => "sudo apt install wine64 wine32 winetricks".to_string(),
        Distro::Arch => "sudo pacman -S wine winetricks".to_string(),
        Distro::OpenSuse => "sudo zypper install wine winetricks wine-32bit".to_string(),
        Distro::Unknown(_) => "Install wine via your package manager".to_string(),
    }
}

fn winetricks_fix(distro: &Distro) -> String {
    match distro {
        Distro::Fedora => "sudo dnf install winetricks".to_string(),
        Distro::Arch => "sudo pacman -S winetricks".to_string(),
        _ => "curl -L https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks | sudo tee /usr/local/bin/winetricks && sudo chmod +x /usr/local/bin/winetricks".to_string(),
    }
}

fn vulkan_fix(distro: &Distro) -> String {
    match distro {
        Distro::Fedora => "sudo dnf install vulkan-tools mesa-vulkan-drivers".to_string(),
        Distro::Debian => "sudo apt install vulkan-tools mesa-vulkan-drivers".to_string(),
        Distro::Arch => "sudo pacman -S vulkan-tools vulkan-icd-loader".to_string(),
        Distro::OpenSuse => "sudo zypper install vulkan-tools".to_string(),
        Distro::Unknown(_) => "Install vulkan-tools and GPU drivers".to_string(),
    }
}

fn gpu_driver_fix(distro: &Distro) -> String {
    match distro {
        Distro::Fedora => "sudo dnf install mesa-dri-drivers".to_string(),
        Distro::Debian => "sudo apt install mesa-utils".to_string(),
        _ => "Install your GPU's Vulkan driver".to_string(),
    }
}

fn gamemode_fix(distro: &Distro) -> String {
    match distro {
        Distro::Fedora => "sudo dnf install gamemode".to_string(),
        Distro::Debian => "sudo apt install gamemode".to_string(),
        Distro::Arch => "sudo pacman -S gamemode".to_string(),
        Distro::OpenSuse => "sudo zypper install gamemode".to_string(),
        Distro::Unknown(_) => "Install gamemode via your package manager".to_string(),
    }
}

fn nvidia_fix(distro: &Distro) -> String {
    match distro {
        Distro::Fedora => "sudo dnf install nvidia-driver-libs".to_string(),
        Distro::Debian => "sudo apt install nvidia-driver".to_string(),
        Distro::Arch => "sudo pacman -S nvidia-utils".to_string(),
        _ => "Install NVIDIA Vulkan driver libraries".to_string(),
    }
}
