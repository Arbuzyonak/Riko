use crate::progress::ProgressSink;
use crate::{RikoError, USER_AGENT};
use futures_util::StreamExt;
use std::io::Write;
use std::path::Path;

const PROGRESS_CHUNK: u64 = 262_144;

pub fn client() -> Result<reqwest::Client, RikoError> {
    Ok(reqwest::Client::builder().user_agent(USER_AGENT).build()?)
}

pub async fn fetch_github_release(
    client: &reqwest::Client,
    repo: &str,
    asset_suffix: &str,
) -> Result<(String, String), RikoError> {
    let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let body: serde_json::Value = client
        .get(&api_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json()
        .await?;

    let tag = body["tag_name"]
        .as_str()
        .ok_or_else(|| RikoError::Other("no tag_name in GitHub release".into()))?
        .to_string();

    let download_url = body["assets"]
        .as_array()
        .ok_or_else(|| RikoError::Other("no assets in GitHub release".into()))?
        .iter()
        .find_map(|a| {
            let name = a["name"].as_str()?;
            if name.ends_with(asset_suffix) {
                a["browser_download_url"].as_str().map(str::to_string)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            RikoError::Other(format!(
                "no asset with suffix '{asset_suffix}' in release {tag}"
            ))
        })?;

    Ok((tag, download_url))
}

pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    stage: &str,
    sink: &dyn ProgressSink,
) -> Result<(), RikoError> {
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(RikoError::Other(format!(
            "download failed: {}",
            resp.status()
        )));
    }

    let total = resp.content_length();
    let mut file = std::fs::File::create(dest)?;
    let mut stream = resp.bytes_stream();
    let mut done: u64 = 0;
    let mut last_emit: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        done += chunk.len() as u64;
        if done - last_emit >= PROGRESS_CHUNK {
            last_emit = done;
            sink.progress(stage, done, total);
        }
    }
    sink.progress(stage, done, total);
    Ok(())
}

pub async fn download_to_memory(
    resp: reqwest::Response,
    stage: &str,
    sink: &dyn ProgressSink,
) -> Result<Vec<u8>, RikoError> {
    let total = resp.content_length();
    let mut bytes: Vec<u8> = Vec::with_capacity(total.unwrap_or(10_000_000) as usize);
    let mut stream = resp.bytes_stream();
    let mut last_emit: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        bytes.extend_from_slice(&chunk);
        let done = bytes.len() as u64;
        if done - last_emit >= PROGRESS_CHUNK {
            last_emit = done;
            sink.progress(stage, done, total);
        }
    }
    sink.progress(stage, bytes.len() as u64, total);
    Ok(bytes)
}
