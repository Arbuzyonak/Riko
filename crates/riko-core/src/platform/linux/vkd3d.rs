use super::dll;
use crate::config::Config;
use crate::progress::ProgressSink;
use crate::RikoError;
use std::path::Path;

const REPO: &str = "HansKristian-Work/vkd3d-proton";
const STAGE: &str = "vkd3d";
const DLLS: &[&str] = &["d3d12.dll", "d3d12core.dll"];
const OVERRIDES: &[(&str, &str)] = &[("d3d12", "native"), ("d3d12core", "native")];

pub async fn install(prefix: &Path, sink: &dyn ProgressSink) -> Result<(), RikoError> {
    sink.started(STAGE, "Installing vkd3d-proton");

    let client = crate::net::client()?;
    let (version, url) = crate::net::fetch_github_release(&client, REPO, ".tar.zst").await?;
    sink.info(STAGE, &format!("vkd3d-proton {version} - downloading"));

    let tmp_dir = Config::data_dir().join("tmp");
    std::fs::create_dir_all(&tmp_dir)?;
    let archive = tmp_dir.join("vkd3d-proton.tar.zst");

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
    use tar::Archive;

    let extract_dir = Config::data_dir().join("tmp").join("vkd3d-extract");
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)?;
    }
    std::fs::create_dir_all(&extract_dir)?;

    let file = std::fs::File::open(archive)?;
    let decoder = zstd::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| RikoError::Other(e.to_string()))?;
    let mut ar = Archive::new(decoder);
    ar.unpack(&extract_dir)
        .map_err(|e| RikoError::Other(e.to_string()))?;

    let sys32 = prefix.join("drive_c/windows/system32");
    let syswow64 = prefix.join("drive_c/windows/syswow64");
    std::fs::create_dir_all(&sys32)?;
    std::fs::create_dir_all(&syswow64)?;

    for entry in std::fs::read_dir(&extract_dir)? {
        let vkd3d_dir = entry?.path();
        dll::install_dlls_from(&vkd3d_dir.join("x64"), &sys32, DLLS)?;
        dll::install_dlls_from(&vkd3d_dir.join("x86"), &syswow64, DLLS)?;
    }

    std::fs::remove_dir_all(&extract_dir).ok();
    Ok(())
}
