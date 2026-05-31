use std::fmt::Display;
use std::{error, fmt};
use thiserror::Error;

/// Error returned when a variant lookup fails.
#[derive(Debug)]
pub struct VariantNotFound {
    pub(crate) v: String,
}

impl error::Error for VariantNotFound {}

impl Display for VariantNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VariantNotFound: {}", &self.v)
    }
}

/// Errors that can occur during mod download or installation.
#[derive(Debug, Error)]
pub enum ModError {
    /// The mod download request failed.
    #[error("Failed to download the mod! Check logs!")]
    DownloadFailed,

    /// The provided mod URL is not valid.
    #[error("Invalid mod URL")]
    InvalidUrl,

    /// A required directory could not be created.
    #[error("Directory creation error: {0}")]
    DirectoryCreationError(String),

    /// An error occurred while extracting the mod archive.
    #[error("Extraction error: {0}")]
    ExtractionError(String),

    /// The staging location path is not a valid file.
    #[error("Invalid staging location")]
    InvalidStagingLocation,

    /// A file could not be opened for reading.
    #[error("File open error: {0}")]
    FileOpenError(String),

    /// An error occurred while reading a ZIP archive.
    #[error("Zip archive error: {0}")]
    ZipArchiveError(String),

    /// The HTTP download request returned an error.
    #[error("Download error: {0}")]
    DownloadError(String),

    /// A destination file could not be created.
    #[error("File creation error: {0}")]
    FileCreateError(String),

    /// A file or directory could not be moved to its destination.
    #[error("File move error: {0}")]
    FileMoveError(String),

    /// A temporary working directory could not be created.
    #[error("Temporary directory creation error: {0}")]
    TempDirCreationError(String),

    /// The mod manifest file could not be deserialized.
    #[error("Failed to deserialize manifest file: {0}")]
    ManifestDeserializeError(String),
}
