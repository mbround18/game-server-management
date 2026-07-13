use std::fmt::Display;
use std::{error, fmt};
use thiserror::Error;

#[derive(Debug)]
pub struct VariantNotFound {
    pub(crate) v: String,
}

impl error::Error for VariantNotFound {}

impl Display for VariantNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VariantNotFound: {}", self.v)
    }
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("Failed to download the mod! Check logs!")]
    DownloadFailed,

    #[error("Invalid mod URL")]
    InvalidUrl,

    #[error("Directory creation error: {0}")]
    DirectoryCreationError(String),

    #[error("Extraction error: {0}")]
    ExtractionError(String),

    #[error("Invalid staging location")]
    InvalidStagingLocation,

    #[error("File open error: {0}")]
    FileOpenError(String),

    #[error("Zip archive error: {0}")]
    ZipArchiveError(String),

    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("File creation error: {0}")]
    FileCreateError(String),

    #[error("File move error: {0}")]
    FileMoveError(String),

    #[error("Temporary directory creation error: {0}")]
    TempDirCreationError(String),

    #[error("Failed to deserialize manifest file: {0}")]
    ManifestDeserializeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_not_found_display() {
        let err = VariantNotFound {
            v: "SomeVariant".to_owned(),
        };
        assert_eq!(err.to_string(), "VariantNotFound: SomeVariant");
    }

    #[test]
    fn mod_error_display_messages() {
        assert_eq!(
            ModError::DownloadFailed.to_string(),
            "Failed to download the mod! Check logs!"
        );
        assert_eq!(ModError::InvalidUrl.to_string(), "Invalid mod URL");
        assert_eq!(
            ModError::InvalidStagingLocation.to_string(),
            "Invalid staging location"
        );
        assert_eq!(
            ModError::DirectoryCreationError("dir".to_owned()).to_string(),
            "Directory creation error: dir"
        );
        assert_eq!(
            ModError::ExtractionError("ext".to_owned()).to_string(),
            "Extraction error: ext"
        );
        assert_eq!(
            ModError::FileOpenError("open".to_owned()).to_string(),
            "File open error: open"
        );
        assert_eq!(
            ModError::ZipArchiveError("zip".to_owned()).to_string(),
            "Zip archive error: zip"
        );
        assert_eq!(
            ModError::DownloadError("dl".to_owned()).to_string(),
            "Download error: dl"
        );
        assert_eq!(
            ModError::FileCreateError("create".to_owned()).to_string(),
            "File creation error: create"
        );
        assert_eq!(
            ModError::FileMoveError("mv".to_owned()).to_string(),
            "File move error: mv"
        );
        assert_eq!(
            ModError::TempDirCreationError("tmp".to_owned()).to_string(),
            "Temporary directory creation error: tmp"
        );
        assert_eq!(
            ModError::ManifestDeserializeError("bad".to_owned()).to_string(),
            "Failed to deserialize manifest file: bad"
        );
    }
}
