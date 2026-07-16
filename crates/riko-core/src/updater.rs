use crate::config::Config;
use crate::progress::ProgressSink;
use crate::{RikoError, VORTEX_BASE};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

const STAGE: &str = "download-vortex";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct RemoteMeta {
    etag: Option<String>,
    last_modified: Option<String>,
    content_length: Option<u64>,
}

impl RemoteMeta {
    fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        let text = |name: &str| {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        Self {
            etag: text("etag"),
            last_modified: text("last-modified"),
            content_length: text("content-length").and_then(|v| v.parse().ok()),
        }
    }

    fn is_empty(&self) -> bool {
        self.etag.is_none() && self.last_modified.is_none() && self.content_length.is_none()
    }
}

fn meta_path() -> PathBuf {
    Config::data_dir().join("vortex-meta.json")
}

fn load_meta() -> Option<RemoteMeta> {
    serde_json::from_str(&std::fs::read_to_string(meta_path()).ok()?).ok()
}

fn save_meta(meta: &RemoteMeta) {
    if meta.is_empty() {
        return;
    }
    if let Ok(json) = serde_json::to_string(meta) {
        std::fs::create_dir_all(Config::data_dir()).ok();
        std::fs::write(meta_path(), json).ok();
    }
}

pub async fn update_if_stale(
    dest: &Path,
    session_token: Option<&str>,
    sink: &dyn ProgressSink,
) -> Result<bool, RikoError> {
    if !dest.exists() {
        return Ok(false);
    }
    let mut req = crate::net::shared().head(format!("{VORTEX_BASE}/download/windows"));
    if let Some(token) = session_token {
        req = req.header("Cookie", format!("session_token={token}"));
    }
    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(false);
    }
    let remote = RemoteMeta::from_headers(resp.headers());
    if remote.is_empty() {
        return Ok(false);
    }
    match load_meta() {
        None => {
            save_meta(&remote);
            Ok(false)
        }
        Some(stored) if stored == remote => Ok(false),
        Some(_) => {
            download_vortex(dest, session_token, sink).await?;
            Ok(true)
        }
    }
}

pub async fn download_vortex(
    dest: &Path,
    session_token: Option<&str>,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let url = format!("{VORTEX_BASE}/download/windows");
    let mut req = crate::net::downloader().get(&url);
    if let Some(token) = session_token {
        req = req.header("Cookie", format!("session_token={token}"));
    }

    sink.started(STAGE, "Downloading Vortex client");
    sink.info(STAGE, &format!("Downloading from {url}"));
    let resp = req.send().await?;

    if !resp.status().is_success() {
        let err = RikoError::Network(resp.error_for_status().unwrap_err());
        sink.finished(STAGE, false, Some(err.to_string()));
        return Err(err);
    }

    let meta = RemoteMeta::from_headers(resp.headers());
    let zip_bytes = crate::net::download_to_memory(resp, STAGE, sink, 1_024 * 1_024 * 1_024).await?;

    sink.info(STAGE, "Extracting Vortex.exe");
    extract_exe_from_zip(&zip_bytes, dest, sink)?;
    save_meta(&meta);
    sink.finished(STAGE, true, Some(dest.display().to_string()));
    Ok(())
}

fn extract_exe_from_zip(
    zip_bytes: &[u8],
    dest: &Path,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| RikoError::Io(std::io::Error::other(e)))?;

    let exe_index = (0..archive.len()).find(|&i| {
        archive
            .by_index(i)
            .map(|f| {
                let name = f.name().to_lowercase();
                name.ends_with(".exe") && (name.contains("vortex") || name.contains("/vortex"))
            })
            .unwrap_or(false)
    });

    let index = exe_index.ok_or_else(|| {
        RikoError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no .exe found inside zip",
        ))
    })?;

    let mut file = archive
        .by_index(index)
        .map_err(|e| RikoError::Io(std::io::Error::other(e)))?;

    sink.info(STAGE, &format!("Found {}", file.name()));

    let tmp_path = dest.with_extension("exe.tmp");
    let mut out = std::fs::File::create(&tmp_path)?;
    std::io::copy(&mut file, &mut out)?;
    out.flush()?;
    drop(out);

    std::fs::rename(&tmp_path, dest)?;
    Ok(())
}
