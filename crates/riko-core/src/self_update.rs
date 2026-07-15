use crate::{net, RikoError};
use serde::Serialize;

pub const RIKO_REPO: &str = "Arbuzik/riko-launcher";

#[derive(Clone, Debug, Serialize)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub release_url: String,
}

pub async fn check() -> Result<Option<UpdateInfo>, RikoError> {
    let url = format!("https://api.github.com/repos/{RIKO_REPO}/releases/latest");
    let body: serde_json::Value = net::shared()
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let latest_tag = body["tag_name"]
        .as_str()
        .ok_or_else(|| RikoError::Other("no tag_name in latest release".into()))?;
    let release_url = body["html_url"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| format!("https://github.com/{RIKO_REPO}/releases"));

    let current = env!("CARGO_PKG_VERSION").to_string();
    if is_newer(latest_tag, &current) {
        Ok(Some(UpdateInfo {
            current,
            latest: strip_v(latest_tag).to_string(),
            release_url,
        }))
    } else {
        Ok(None)
    }
}

fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

fn parts(version: &str) -> Vec<u64> {
    strip_v(version)
        .split(['.', '-', '+'])
        .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

fn is_newer(candidate: &str, current: &str) -> bool {
    let (a, b) = (parts(candidate), parts(current));
    let len = a.len().max(b.len());
    for i in 0..len {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("v0.1.0", "0.2.0"));
        assert!(is_newer("v0.2.0-rc1", "0.1.9"));
        assert!(!is_newer("0.1.0", "0.1.0-beta"));
    }
}
