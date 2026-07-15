use crate::config::Config;
use crate::progress::ProgressSink;
use crate::RikoError;
use serde::Serialize;
use std::path::PathBuf;

const RELEASES_URL: &str =
    "https://api.github.com/repos/Kron4ek/Wine-Builds/releases?per_page=5";
const STAGE: &str = "wine-install";

#[derive(Clone, Debug, Serialize)]
pub struct AvailableWine {
    pub name: String,
    pub download_url: String,
    pub size_mb: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct InstalledWine {
    pub name: String,
    pub wine_binary: String,
}

pub fn wine_dir() -> PathBuf {
    Config::data_dir().join("wine")
}

pub fn list_installed() -> Vec<InstalledWine> {
    let mut installed = vec![];
    if let Ok(entries) = std::fs::read_dir(wine_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            let binary = path.join("bin/wine");
            if binary.is_file()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                installed.push(InstalledWine {
                    name: name.to_string(),
                    wine_binary: binary.display().to_string(),
                });
            }
        }
    }
    installed.sort_by(|a, b| b.name.cmp(&a.name));
    installed
}

pub async fn list_available() -> Result<Vec<AvailableWine>, RikoError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(RELEASES_URL)
        .header("User-Agent", "riko-launcher")
        .send()
        .await?
        .error_for_status()?;
    let releases: Vec<serde_json::Value> = resp.json().await?;
    let mut available = vec![];
    for release in &releases {
        let Some(assets) = release.get("assets").and_then(|a| a.as_array()) else {
            continue;
        };
        for asset in assets {
            let Some(name) = asset.get("name").and_then(|n| n.as_str()) else {
                continue;
            };
            if !name.starts_with("wine-") || !name.ends_with("-amd64.tar.xz") {
                continue;
            }
            let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str()) else {
                continue;
            };
            available.push(AvailableWine {
                name: name
                    .trim_end_matches("-amd64.tar.xz")
                    .to_string(),
                download_url: url.to_string(),
                size_mb: asset.get("size").and_then(|s| s.as_u64()).unwrap_or(0) / 1_000_000,
            });
        }
    }
    Ok(available)
}

#[cfg(unix)]
pub async fn install(url: &str, sink: &dyn ProgressSink) -> Result<InstalledWine, RikoError> {
    sink.started(STAGE, "Downloading wine build");
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "riko-launcher")
        .send()
        .await?
        .error_for_status()?;
    let bytes = crate::net::download_to_memory(resp, STAGE, sink).await?;

    sink.info(STAGE, "Extracting");
    let dest = wine_dir();
    std::fs::create_dir_all(&dest)?;
    let before = installed_names();
    let result = extract_tar_xz(&bytes, &dest);
    if let Err(e) = result {
        sink.finished(STAGE, false, Some(e.to_string()));
        return Err(e);
    }

    let created = installed_names()
        .into_iter()
        .find(|name| !before.contains(name))
        .ok_or_else(|| RikoError::Other("archive did not contain a wine build".to_string()))?;
    let installed = InstalledWine {
        wine_binary: dest.join(&created).join("bin/wine").display().to_string(),
        name: created,
    };
    sink.finished(STAGE, true, Some(installed.name.clone()));
    Ok(installed)
}

#[cfg(not(unix))]
pub async fn install(_url: &str, _sink: &dyn ProgressSink) -> Result<InstalledWine, RikoError> {
    Err(RikoError::Other(
        "wine builds are only used on Linux".to_string(),
    ))
}

pub fn remove(name: &str) -> Result<(), RikoError> {
    let safe = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_');
    if !safe {
        return Err(RikoError::Other(format!("invalid wine build name '{name}'")));
    }
    let dir = wine_dir().join(name);
    if !dir.join("bin/wine").is_file() {
        return Err(RikoError::Other(format!("'{name}' is not an installed wine build")));
    }
    std::fs::remove_dir_all(dir)?;
    Ok(())
}

fn installed_names() -> Vec<String> {
    list_installed().into_iter().map(|w| w.name).collect()
}

#[cfg(unix)]
fn extract_tar_xz(bytes: &[u8], dest: &std::path::Path) -> Result<(), RikoError> {
    let decoder = xz2::read::XzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest)
        .map_err(|e| RikoError::Other(format!("failed to extract wine build: {e}")))?;
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extracts_wine_build_layout() {
        let scratch = std::env::temp_dir().join(format!("riko-wine-test-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();

        let mut tar_bytes = vec![];
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let content = b"#!/bin/sh\n";
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, "wine-9.0-test/bin/wine", content.as_slice())
                .unwrap();
            builder.finish().unwrap();
        }
        let mut xz_bytes = vec![];
        {
            let mut encoder = xz2::write::XzEncoder::new(&mut xz_bytes, 1);
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }

        extract_tar_xz(&xz_bytes, &scratch).unwrap();
        assert!(scratch.join("wine-9.0-test/bin/wine").is_file());
        std::fs::remove_dir_all(&scratch).unwrap();
    }
}
