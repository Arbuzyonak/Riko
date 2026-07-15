use super::dll;
use crate::config::Config;
use crate::progress::ProgressSink;
use crate::RikoError;
use std::path::Path;

const REPO: &str = "doitsujin/dxvk";
const STAGE: &str = "dxvk";
const DLLS: &[&str] = &["d3d9.dll", "d3d10core.dll", "d3d10_1.dll", "d3d11.dll", "dxgi.dll"];
const OVERRIDES: &[(&str, &str)] = &[
    ("d3d9", "native,builtin"),
    ("d3d10core", "native,builtin"),
    ("d3d10_1", "native,builtin"),
    ("d3d11", "native,builtin"),
    ("dxgi", "native,builtin"),
];

pub async fn install(prefix: &Path, sink: &dyn ProgressSink) -> Result<(), RikoError> {
    sink.started(STAGE, "Installing DXVK");

    let client = crate::net::client()?;
    let (version, url) = crate::net::fetch_github_release(&client, REPO, ".tar.gz").await?;
    sink.info(STAGE, &format!("DXVK {version} - downloading"));

    let tmp_dir = Config::data_dir().join("tmp");
    std::fs::create_dir_all(&tmp_dir)?;
    let archive = tmp_dir.join("dxvk.tar.gz");

    crate::net::download_file(&client, &url, &archive, STAGE, sink).await?;
    extract_and_install(&archive, prefix)?;
    std::fs::remove_file(&archive).ok();

    for (name, override_type) in OVERRIDES {
        dll::set_dll_override(prefix, name, override_type);
    }

    sink.finished(STAGE, true, Some(version));
    Ok(())
}

fn extract_and_install(archive: &Path, prefix: &Path) -> Result<(), RikoError> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let extract_dir = Config::data_dir().join("tmp").join("dxvk-extract");
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)?;
    }
    std::fs::create_dir_all(&extract_dir)?;

    let file = std::fs::File::open(archive)?;
    let gz = GzDecoder::new(std::io::BufReader::new(file));
    let mut ar = Archive::new(gz);
    ar.unpack(&extract_dir)
        .map_err(|e| RikoError::Other(e.to_string()))?;

    let sys32 = prefix.join("drive_c/windows/system32");
    let syswow64 = prefix.join("drive_c/windows/syswow64");
    std::fs::create_dir_all(&sys32)?;
    std::fs::create_dir_all(&syswow64)?;

    for entry in std::fs::read_dir(&extract_dir)? {
        let dxvk_dir = entry?.path();
        dll::install_dlls_from(&dxvk_dir.join("x64"), &sys32, DLLS)?;
        dll::install_dlls_from(&dxvk_dir.join("x32"), &syswow64, DLLS)?;
    }

    std::fs::remove_dir_all(&extract_dir).ok();
    Ok(())
}
