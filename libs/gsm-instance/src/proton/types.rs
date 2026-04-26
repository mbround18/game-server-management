use std::path::PathBuf;
use thiserror::Error;

/// ProtonVersion represents a specific Proton version
#[derive(Debug, Clone)]
pub struct ProtonVersion {
    /// The name of the Proton version (e.g. "GE-Proton10-1")
    pub name: String,
    /// The full path to the Proton executable
    pub path: PathBuf,
    /// The directory containing the Proton installation
    pub dir: PathBuf,
}

/// ProtonRelease represents a release from the GloriousEggroll repository
#[derive(Debug, Clone, PartialEq)]
pub struct ProtonRelease {
    /// The version tag (e.g. "GE-Proton10-1")
    pub tag: String,
    /// The download URL for the release
    pub download_url: String,
    /// The release date
    pub release_date: String,
}

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Failed to parse version: {0}")]
    ParseError(String),

    #[error("Version not found: {0}")]
    NotFound(String),
}

/// Parse a Proton version string
pub fn parse_version(version_str: &str) -> Result<String, VersionError> {
    // Handle GE-Proton special case
    if version_str.starts_with("GE-") {
        return Ok(version_str.to_string());
    }

    // Try to match version format like "10-1" to "GE-Proton10-1"
    if version_str.contains('-') {
        return Ok(format!("GE-Proton{}", version_str));
    }

    // Simple version number like "10" to "GE-Proton10-1"
    if version_str.parse::<u32>().is_ok() {
        return Ok(format!("GE-Proton{}-1", version_str));
    }

    // If it's a full path to proton, use it as is
    if version_str.ends_with("/proton") || version_str.contains("proton") {
        return Ok(version_str.to_string());
    }

    Err(VersionError::ParseError(format!(
        "Unrecognized version format: {}",
        version_str
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_full() {
        assert_eq!(parse_version("GE-Proton10-1").unwrap(), "GE-Proton10-1");
    }

    #[test]
    fn test_parse_version_short() {
        assert_eq!(parse_version("10-1").unwrap(), "GE-Proton10-1");
    }

    #[test]
    fn test_parse_version_number_only() {
        assert_eq!(parse_version("10").unwrap(), "GE-Proton10-1");
    }

    #[test]
    fn test_parse_version_path() {
        assert_eq!(parse_version("/path/to/proton").unwrap(), "/path/to/proton");
    }

    #[test]
    fn test_parse_version_invalid() {
        assert!(parse_version("invalid").is_err());
    }
}
