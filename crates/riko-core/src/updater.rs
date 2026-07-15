use crate::progress::ProgressSink;
use crate::{RikoError, VORTEX_BASE};
use std::io::{Cursor, Write};
use std::path::Path;

const STAGE: &str = "download-vortex";

pub async fn download_vortex(
    dest: &Path,
    session_token: Option<&str>,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let url = format!("{VORTEX_BASE}/download/windows");
    let client = reqwest::Client::new();
    let mut req = client.get(&url);
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

    let zip_bytes = crate::net::download_to_memory(resp, STAGE, sink).await?;

    sink.info(STAGE, "Extracting Vortex.exe");
    extract_exe_from_zip(&zip_bytes, dest, sink)?;
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
