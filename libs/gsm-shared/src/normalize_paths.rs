use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use walkdir::WalkDir;

/// Replaces backslashes with forward slashes in the string representation of a path.
fn normalize_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy().replace('\\', "/");
    PathBuf::from(path_str)
}

/// Validates that the source directory exists.
fn validate_source_dir(src_dir: &Path) -> std::io::Result<()> {
    if !src_dir.is_dir() {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory {:?} does not exist", src_dir),
        ))
    } else {
        Ok(())
    }
}

/// Moves and normalizes contents from the source directory into the temporary directory.
/// Each file or subdirectory is moved to a new location whose relative path is normalized.
fn move_and_normalize_to_temp(src_dir: &Path, temp_root: &Path) -> std::io::Result<()> {
    for entry in WalkDir::new(src_dir).into_iter().filter_map(Result::ok) {
        let src_path = entry.path();
        // Compute the relative path to src_dir.
        let relative_path = src_path
            .strip_prefix(src_dir)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        // Normalize the relative path by replacing backslashes with forward slashes.
        let normalized_relative_path = normalize_path(relative_path);
        let temp_dest_path = temp_root.join(&normalized_relative_path);

        if src_path.is_dir() {
            fs::create_dir_all(&temp_dest_path)?;
        } else {
            if let Some(parent) = temp_dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(src_path, &temp_dest_path)?;
        }
    }
    Ok(())
}

/// Moves normalized contents from the temporary directory back into the source directory.
fn move_normalized_back(temp_root: &Path, src_dir: &Path) -> std::io::Result<()> {
    for entry in WalkDir::new(temp_root).into_iter().filter_map(Result::ok) {
        let entry_path = entry.path();
        let relative_path = entry_path
            .strip_prefix(temp_root)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let original_dest_path = src_dir.join(relative_path);

        if entry_path.is_dir() {
            fs::create_dir_all(&original_dest_path)?;
        } else {
            if let Some(parent) = original_dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(entry_path, &original_dest_path)?;
        }
    }
    Ok(())
}

/// Normalizes the paths within `src_dir` by moving its contents to a temporary location,
/// normalizing each relative path (replacing backslashes with forward slashes),
/// and then moving the normalized contents back into a fresh `src_dir`.
///
/// # Errors
///
/// Returns an error if any file system operation fails.
pub fn normalize_paths(src_dir: &Path) -> std::io::Result<()> {
    // Ensure the source directory exists.
    validate_source_dir(src_dir)?;

    // Create a temporary directory.
    let temp_dir = tempdir()?;
    let temp_root = temp_dir.path();

    // Move and normalize contents from src_dir to the temporary directory.
    move_and_normalize_to_temp(src_dir, temp_root)?;

    // Remove the original directory and recreate it empty.
    fs::remove_dir_all(src_dir)?;
    fs::create_dir_all(src_dir)?;

    // Move the normalized contents back into the source directory.
    move_normalized_back(temp_root, src_dir)?;

    // temp_dir is automatically deleted when it goes out of scope.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;
    use walkdir::WalkDir;

    /// Creates a test directory structure under `base_dir`.
    /// Specifically, creates a directory with a backslash in its name (on systems where this is allowed)
    /// and writes a test file within it.
    fn setup_test_dir(base_dir: &Path) -> std::io::Result<PathBuf> {
        let src_dir = base_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // Create a directory with a backslash in its name.
        // On Unix, '\' is just another character.
        let problematic_dir = src_dir.join("foo\\bar");
        fs::create_dir_all(&problematic_dir)?;

        // Create a test file inside the problematic directory.
        let file_path = problematic_dir.join("test.txt");
        let mut file = fs::File::create(&file_path)?;
        writeln!(file, "Hello, world!")?;
        Ok(src_dir)
    }

    /// Checks that none of the file paths under `dir` contain a backslash.
    fn assert_no_backslashes(dir: &Path) {
        for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
            let path_str = entry.path().to_string_lossy();
            assert!(
                !path_str.contains('\\'),
                "Path {} still contains a backslash",
                path_str
            );
        }
    }

    #[test]
    fn test_normalize_paths_removes_backslashes() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = setup_test_dir(temp_dir.path())?;
        // Call normalize_paths on the source directory.
        normalize_paths(&src_dir)?;

        // Assert that the normalized directory structure contains no backslashes.
        assert_no_backslashes(&src_dir);

        // Optionally, check that the test file exists with its content preserved.
        let normalized_file = src_dir.join("foo").join("bar").join("test.txt");
        let content = fs::read_to_string(normalized_file)?;
        assert_eq!(content.trim(), "Hello, world!");
        Ok(())
    }

    #[test]
    fn test_normalize_paths_errors_on_nonexistent_dir() {
        let non_existent = PathBuf::from("this_directory_should_not_exist");
        let result = normalize_paths(&non_existent);
        assert!(result.is_err());
    }
}
