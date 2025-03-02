use flate2::Compression;
use flate2::write::GzEncoder;
use glob::glob;
use log::{debug, error, info};
use std::fs::{File, remove_file};
use std::io::{Error as IoError, ErrorKind};
use std::path::Path;
use tar::Builder;
use thiserror::Error;

/// Custom error type for backup failures.
#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Failed to create backup file at {0}")]
    CreateBackupError(String),
    #[error("Glob pattern error: {0}")]
    GlobPatternError(#[from] glob::PatternError),
    #[error("Error reading glob entry: {0}")]
    GlobEntryError(#[from] glob::GlobError),
    #[error("Tar archive error: {0}")]
    TarError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] IoError),
}

/// Creates a compressed tar archive (tar.gz) backup of all files under the input directory.
///
/// # Parameters
///
/// - `input`: The directory to backup (all files matching `input/**/*` will be included).
/// - `output`: The path for the output tar.gz archive.
///
/// # Behavior
///
/// - Files whose paths contain the substring `"backup_auto"` are skipped.
/// - The backup file is created using a gzip encoder with default compression.
///
/// # Errors
///
/// Returns a `BackupError` if:
/// - The input directory does not exist or is not a directory.
/// - The output file cannot be created.
/// - A glob pattern cannot be read.
/// - Any file cannot be added to the tar archive.
/// - The archive cannot be finished.
///
/// # Examples
///
/// ```rust,no_run
/// # // Note: This example is marked `no_run` because it requires the "data" directory to exist.
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // Backup the directory "data" into "backup.tar.gz"
/// gsm_backup::backup("data", "backup.tar.gz")?;
/// # Ok(())
/// # }
/// ```
pub fn backup<P, Q>(input: P, output: Q) -> Result<(), BackupError>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let input = input.as_ref();
    let output = output.as_ref();

    // Check that input exists and is a directory.
    if !input.exists() || !input.is_dir() {
        return Err(BackupError::IoError(IoError::new(
            ErrorKind::NotFound,
            format!(
                "Input directory {:?} does not exist or is not a directory",
                input
            ),
        )));
    }

    debug!("Creating archive of {:?}", input);
    debug!("Output set to {:?}", output);

    // Attempt to create the output backup file.
    let tar_gz = File::create(output)
        .map_err(|_| BackupError::CreateBackupError(output.display().to_string()))?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    // Build a glob pattern for all files and directories under the input.
    let pattern = format!("{}/**/*", input.display());
    let entries = glob(&pattern).expect("Failed to read glob pattern");

    for entry in entries {
        match entry {
            Ok(path) => {
                let path_str = path.display().to_string();
                // Skip files whose names contain "backup_auto"
                if path_str.contains("backup_auto") {
                    continue;
                }
                // Compute the relative path from the input directory.
                let relative = path.strip_prefix(input).unwrap_or(&path);
                info!(
                    "Adding {} to backup file, with relative path {:?}",
                    path_str, relative
                );
                if let Err(err) = tar.append_path_with_name(&path, relative) {
                    error!("Failed to add {} to backup file", path_str);
                    error!("{:?}", err);
                    let _ = remove_file(output);
                    return Err(BackupError::TarError(err.to_string()));
                } else {
                    debug!("Successfully added {} to backup file", path_str);
                }
            }
            Err(e) => error!("Error reading glob entry: {:?}", e),
        }
    }
    tar.finish()
        .map_err(|e| BackupError::TarError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tar::Archive;
    use tempfile::{NamedTempFile, tempdir};

    /// Sets up a temporary directory with a few test files and subdirectories.
    fn setup_test_dir() -> tempfile::TempDir {
        let dir = tempdir().expect("Failed to create temp dir for test");
        // Create a file "foo.txt" in the directory.
        let file_path = dir.path().join("foo.txt");
        fs::write(&file_path, "hello world").expect("Failed to write test file");
        // Create a subdirectory "sub" and a file inside it.
        let sub_dir = dir.path().join("sub");
        fs::create_dir_all(&sub_dir).expect("Failed to create subdirectory");
        let sub_file_path = sub_dir.join("bar.txt");
        fs::write(&sub_file_path, "subdirectory file").expect("Failed to write subdirectory file");
        // Create a file that should be skipped.
        let skip_file = dir.path().join("backup_auto_skip.txt");
        fs::write(&skip_file, "should be skipped").expect("Failed to write skip file");
        dir
    }

    /// Reads the tar.gz archive and returns a vector of relative paths included in it.
    fn read_archive<P: AsRef<Path>>(archive_path: P) -> Vec<String> {
        let file = File::open(archive_path).expect("Failed to open archive file");
        let mut decompressor = flate2::read::GzDecoder::new(file);
        let mut archive = Archive::new(&mut decompressor);
        let mut paths = Vec::new();
        for entry in archive.entries().expect("Failed to read archive entries") {
            let entry = entry.expect("Error reading entry");
            let path = entry.path().expect("Error getting entry path").into_owned();
            paths.push(path.display().to_string());
        }
        paths
    }

    #[test]
    fn test_backup_success() {
        let test_dir = setup_test_dir();
        // Create a temporary output file.
        let backup_file = NamedTempFile::new().expect("Failed to create temp file");
        let backup_path = backup_file.path().to_owned();

        // Run backup on the test directory.
        backup(test_dir.path(), &backup_path).expect("Backup failed");

        // Read archive and verify that "foo.txt" and "sub/bar.txt" are included,
        // but that any file with "backup_auto" in its name is skipped.
        let archived_files = read_archive(&backup_path);
        assert!(archived_files.iter().any(|s| s.contains("foo.txt")));
        assert!(archived_files.iter().any(|s| s.contains("sub/bar.txt")));
        assert!(!archived_files.iter().any(|s| s.contains("backup_auto")));
    }

    #[test]
    fn test_backup_nonexistent_input() {
        let tmp_dir = tempdir().unwrap();
        let nonexistent = tmp_dir.path().join("nonexistent_dir");
        let backup_file = NamedTempFile::new().expect("Failed to create temp file");
        let result = backup(&nonexistent, backup_file.path());
        assert!(result.is_err());
    }
}
