use crate::config::Config;
use crate::progress::ProgressSink;
use crate::{net, RikoError};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub const DEFAULT_INDEX_URL: &str =
    "https://raw.githubusercontent.com/Arbuzik/riko-shaders/main/index.json";

const STAGE: &str = "shader-cache";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShaderEntry {
    pub game_id: u32,
    pub gpu_key: String,
    #[serde(default)]
    pub label: Option<String>,
    pub download_url: String,
    pub sha256: String,
    #[serde(default)]
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Index {
    #[serde(default)]
    entries: Vec<ShaderEntry>,
}

pub fn base_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache")
        })
        .join("vortex-shaders")
}

pub fn dir_for(game_id: u32) -> PathBuf {
    base_dir().join(game_id.to_string())
}

pub fn index_url(cfg: &Config) -> String {
    cfg.shader_cache
        .index_url
        .clone()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_INDEX_URL.to_string())
}

pub fn gpu_key() -> String {
    let raw = detect_gpu().unwrap_or_default();
    let slug = slugify(&raw);
    if slug.is_empty() {
        "unknown-gpu".to_string()
    } else {
        slug
    }
}

fn detect_gpu() -> Option<String> {
    let output = std::process::Command::new("vulkaninfo")
        .arg("--summary")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(eq) = trimmed.find('=')
            && trimmed[..eq].trim() == "deviceName"
        {
            let name = trimmed[eq + 1..].trim().to_lowercase();
            if !name.contains("llvmpipe") && !name.contains("softpipe") {
                return Some(name);
            }
        }
    }
    None
}

fn slugify(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub async fn fetch_index(url: &str) -> Result<Vec<ShaderEntry>, RikoError> {
    let index: Index = net::send_retrying(|| net::shared().get(url), 2)
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(index.entries)
}

pub fn find_match<'a>(
    entries: &'a [ShaderEntry],
    game_id: u32,
    gpu_key: &str,
) -> Option<&'a ShaderEntry> {
    entries
        .iter()
        .find(|e| e.game_id == game_id && e.gpu_key == gpu_key)
}

pub async fn prefetch(cfg: &Config, game_id: u32, sink: &dyn ProgressSink) -> Result<bool, RikoError> {
    if !cfg.shader_cache.community {
        return Ok(false);
    }
    let key = tokio::task::spawn_blocking(gpu_key)
        .await
        .unwrap_or_default();
    let entries = fetch_index(&index_url(cfg)).await?;
    let Some(entry) = find_match(&entries, game_id, &key) else {
        return Ok(false);
    };
    download_and_apply(entry, game_id, sink).await?;
    Ok(true)
}

pub async fn download_and_apply(
    entry: &ShaderEntry,
    game_id: u32,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    sink.started(STAGE, "Fetching community shader cache");
    let resp = net::downloader()
        .get(&entry.download_url)
        .send()
        .await?
        .error_for_status()?;
    let bytes = net::download_to_memory(resp, STAGE, sink, 512 * 1_024 * 1_024).await?;

    let actual = sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(entry.sha256.trim()) {
        sink.finished(STAGE, false, Some("checksum mismatch".to_string()));
        return Err(RikoError::Other(format!(
            "shader cache checksum mismatch: expected {}, got {actual}",
            entry.sha256
        )));
    }

    let dest = dir_for(game_id);
    std::fs::create_dir_all(&dest)?;
    extract_into(&bytes, &dest)?;
    sink.finished(STAGE, true, None);
    Ok(())
}

fn extract_into(bytes: &[u8], dest: &Path) -> Result<(), RikoError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| RikoError::Other(format!("invalid shader archive: {e}")))?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| RikoError::Other(format!("corrupt shader archive: {e}")))?;
        let Some(rel) = file.enclosed_name() else {
            return Err(RikoError::Other("shader archive has an unsafe path".to_string()));
        };
        if rel.as_os_str().is_empty() || file.is_dir() {
            continue;
        }
        let out = dest.join(&rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut writer = std::fs::File::create(&out)?;
        std::io::copy(&mut file, &mut writer)?;
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_gpu_names() {
        assert_eq!(slugify("NVIDIA GeForce RTX 2070 SUPER"), "nvidia-geforce-rtx-2070-super");
        assert_eq!(slugify("AMD Radeon RX 6800 XT (RADV)"), "amd-radeon-rx-6800-xt-radv");
        assert_eq!(slugify("  "), "");
    }

    #[test]
    fn matches_game_and_gpu() {
        let entries = vec![
            ShaderEntry {
                game_id: 3,
                gpu_key: "nvidia-rtx-2070".to_string(),
                label: None,
                download_url: "u".to_string(),
                sha256: "s".to_string(),
                size_bytes: 0,
            },
            ShaderEntry {
                game_id: 3,
                gpu_key: "amd-6800".to_string(),
                label: None,
                download_url: "u2".to_string(),
                sha256: "s2".to_string(),
                size_bytes: 0,
            },
        ];
        assert_eq!(find_match(&entries, 3, "amd-6800").unwrap().download_url, "u2");
        assert!(find_match(&entries, 3, "intel-arc").is_none());
        assert!(find_match(&entries, 9, "amd-6800").is_none());
    }

    #[test]
    fn dir_is_per_game_under_base() {
        assert!(dir_for(7).ends_with("vortex-shaders/7"));
    }
}
