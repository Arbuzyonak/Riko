use crate::progress::ProgressSink;
use crate::{RikoError, USER_AGENT};
use futures_util::StreamExt;
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

const PROGRESS_CHUNK: u64 = 262_144;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub fn shared() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build shared HTTP client")
    })
}

pub fn downloader() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .expect("failed to build downloader HTTP client")
    })
}

pub async fn send_retrying<F>(make: F, attempts: u32) -> Result<reqwest::Response, RikoError>
where
    F: Fn() -> reqwest::RequestBuilder,
{
    let total = attempts.max(1);
    let mut last: Option<reqwest::Error> = None;
    for attempt in 0..total {
        match make().send().await {
            Ok(resp) => return Ok(resp),
            Err(err) => {
                last = Some(err);
                if attempt + 1 < total {
                    tokio::time::sleep(Duration::from_millis(300 * (attempt as u64 + 1))).await;
                }
            }
        }
    }
    Err(RikoError::Network(last.expect("at least one attempt")))
}

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
    max_bytes: u64,
) -> Result<Vec<u8>, RikoError> {
    let total = resp.content_length();
    if let Some(len) = total
        && len > max_bytes
    {
        return Err(RikoError::Other(format!(
            "download too large: {len} bytes exceeds the {max_bytes}-byte limit"
        )));
    }
    let capacity = total.unwrap_or(10_000_000).min(max_bytes) as usize;
    let mut bytes: Vec<u8> = Vec::with_capacity(capacity);
    let mut stream = resp.bytes_stream();
    let mut last_emit: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if bytes.len() as u64 + chunk.len() as u64 > max_bytes {
            return Err(RikoError::Other(format!(
                "download exceeded the {max_bytes}-byte limit"
            )));
        }
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
