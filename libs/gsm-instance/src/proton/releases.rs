use super::types::ProtonRelease;
use reqwest;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReleaseError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Release not found: {0}")]
    NotFound(String),
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize, Debug)]
struct GitHubAsset {
    browser_download_url: String,
    name: String,
}

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases";

pub fn list_available_releases() -> Result<Vec<ProtonRelease>, ReleaseError> {
    let releases: Vec<GitHubRelease> = reqwest::blocking::Client::new()
        .get(GITHUB_API_URL)
        .header("User-Agent", "gsm-instance")
        .send()?
        .json()?;

    let proton_releases = releases
        .into_iter()
        .filter_map(|r| {
            r.assets
                .iter()
                .find(|&a| a.name.ends_with(".tar.gz"))
                .map(|a| ProtonRelease {
                    tag: r.tag_name.clone(),
                    download_url: a.browser_download_url.clone(),
                    release_date: r.published_at.clone(),
                })
        })
        .collect();

    Ok(proton_releases)
}

pub fn fetch_latest_release() -> Result<ProtonRelease, ReleaseError> {
    let releases = list_available_releases()?;
    releases
        .into_iter()
        .next()
        .ok_or_else(|| ReleaseError::NotFound("No releases found".to_string()))
}

pub fn fetch_specific_release(version: &str) -> Result<ProtonRelease, ReleaseError> {
    let releases = list_available_releases()?;
    releases
        .into_iter()
        .find(|r| r.tag == version)
        .ok_or_else(|| ReleaseError::NotFound(format!("Release {} not found", version)))
}
