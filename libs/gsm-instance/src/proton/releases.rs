//! # Proton Releases
//!
//! This module provides functionality for fetching information about Proton GE releases
//! from the GitHub API. It allows listing all available releases, fetching the latest
//! release, or fetching a specific release by version tag.
use super::types::ProtonRelease;
use reqwest;
use serde::Deserialize;
use thiserror::Error;

/// Represents errors that can occur while fetching Proton release information.
#[derive(Error, Debug)]
pub enum ReleaseError {
    /// A network error occurred while making a request to the GitHub API.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// An error occurred while parsing the JSON response from the GitHub API.
    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// The requested release was not found.
    #[error("Release not found: {0}")]
    NotFound(String),
}

/// Represents a release from the GitHub API.
#[derive(Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

/// Represents an asset within a GitHub release.
#[derive(Deserialize, Debug)]
struct GitHubAsset {
    browser_download_url: String,
    name: String,
}

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases";

/// Fetches a list of all available Proton GE releases from GitHub.
///
/// This function makes a request to the GitHub API to get a list of all releases
/// for the `proton-ge-custom` repository. It then filters these releases to find
/// the ones that have a `.tar.gz` asset, which is the format used for distribution.
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

/// Fetches the latest available Proton GE release from GitHub.
pub fn fetch_latest_release() -> Result<ProtonRelease, ReleaseError> {
    let releases = list_available_releases()?;
    releases
        .into_iter()
        .next()
        .ok_or_else(|| ReleaseError::NotFound("No releases found".to_string()))
}

/// Fetches a specific Proton GE release by its version tag.
///
/// # Arguments
///
/// * `version`: The tag of the release to fetch (e.g., "GE-Proton8-25").
pub fn fetch_specific_release(version: &str) -> Result<ProtonRelease, ReleaseError> {
    let releases = list_available_releases()?;
    releases
        .into_iter()
        .find(|r| r.tag == version)
        .ok_or_else(|| ReleaseError::NotFound(format!("Release {} not found", version)))
}
