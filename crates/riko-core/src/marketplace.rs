use crate::config::Config;
use crate::plugin;
use crate::progress::ProgressSink;
use crate::{net, RikoError};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub const DEFAULT_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/Arbuzik/riko-plugins/main/catalog.json";

const STAGE: &str = "marketplace";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: String,
    #[serde(default)]
    pub platforms: Vec<String>,
    pub download_url: String,
    pub sha256: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Catalog {
    #[serde(default)]
    plugins: Vec<CatalogEntry>,
}

pub fn catalog_url(cfg: &Config) -> String {
    cfg.plugins
        .catalog_url
        .clone()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_CATALOG_URL.to_string())
}

pub async fn fetch_catalog(url: &str) -> Result<Vec<CatalogEntry>, RikoError> {
    let catalog: Catalog = net::send_retrying(|| net::shared().get(url), 3)
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(catalog.plugins)
}

pub async fn download_and_extract(
    entry: &CatalogEntry,
    sink: &dyn ProgressSink,
) -> Result<PathBuf, RikoError> {
    let dest = plugin::plugins_dir().join(&entry.name);
    if dest.exists() {
        return Err(RikoError::Plugin(format!(
            "plugin '{}' is already installed",
            entry.name
        )));
    }

    sink.started(STAGE, &format!("Downloading {}", entry.name));
    let resp = net::downloader()
        .get(&entry.download_url)
        .send()
        .await?
        .error_for_status()?;
    let bytes = net::download_to_memory(resp, STAGE, sink, 128 * 1_024 * 1_024).await?;

    let actual = sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(entry.sha256.trim()) {
        sink.finished(STAGE, false, Some("checksum mismatch".to_string()));
        return Err(RikoError::Plugin(format!(
            "checksum mismatch for '{}': expected {}, got {actual}",
            entry.name, entry.sha256
        )));
    }

    sink.info(STAGE, "Verified checksum; extracting");
    if let Err(e) = extract_zip_into(&bytes, &dest, &entry.name) {
        std::fs::remove_dir_all(&dest).ok();
        sink.finished(STAGE, false, Some(e.to_string()));
        return Err(e);
    }

    match plugin::manifest::load(&dest) {
        Ok(m) if m.plugin.name == entry.name => {}
        Ok(m) => {
            std::fs::remove_dir_all(&dest).ok();
            return Err(RikoError::Plugin(format!(
                "archive manifest names '{}' but catalog entry is '{}'",
                m.plugin.name, entry.name
            )));
        }
        Err(e) => {
            std::fs::remove_dir_all(&dest).ok();
            return Err(e);
        }
    }

    sink.finished(STAGE, true, Some(entry.name.clone()));
    Ok(dest)
}

fn extract_zip_into(bytes: &[u8], dest: &Path, name: &str) -> Result<(), RikoError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| RikoError::Plugin(format!("invalid plugin archive: {e}")))?;
    std::fs::create_dir_all(dest)?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| RikoError::Plugin(format!("corrupt archive entry: {e}")))?;
        let Some(enclosed) = file.enclosed_name() else {
            return Err(RikoError::Plugin(
                "archive contains an unsafe path".to_string(),
            ));
        };
        let relative = enclosed
            .strip_prefix(name)
            .unwrap_or(enclosed.as_path())
            .to_path_buf();
        if relative.as_os_str().is_empty() {
            continue;
        }
        let out = dest.join(&relative);
        if file.is_dir() {
            std::fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut writer = std::fs::File::create(&out)?;
        std::io::copy(&mut file, &mut writer)?;
        #[cfg(unix)]
        if file.unix_mode().is_some_and(|m| m & 0o111 != 0) {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o755)).ok();
        }
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
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn catalog_url_falls_back_to_default() {
        let mut cfg = Config::default();
        assert_eq!(catalog_url(&cfg), DEFAULT_CATALOG_URL);
        cfg.plugins.catalog_url = Some("  ".to_string());
        assert_eq!(catalog_url(&cfg), DEFAULT_CATALOG_URL);
        cfg.plugins.catalog_url = Some("https://example.com/c.json".to_string());
        assert_eq!(catalog_url(&cfg), "https://example.com/c.json");
    }
}
