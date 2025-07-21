use crate::constants::SUPPORTED_FILE_TYPES;
use crate::errors::ModError;
use gsm_shared::{
    get_md5_hash, is_valid_url, normalize_paths, parse_file_name, url_parse_file_type,
};

use crate::parse_mod_string::parse_mod_string;
use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use reqwest::Url;
use std::convert::TryFrom;
use std::fs::{File, create_dir_all};
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use tracing::{debug, error};
use walkdir::WalkDir;
use zip::ZipArchive;

pub struct ManagedMod {
    pub(crate) url: String,
    pub(crate) file_type: String,
    pub(crate) staging_location: PathBuf,
    pub(crate) installed: bool,
    pub(crate) downloaded: bool,
    pub(crate) game_directory: PathBuf,
    pub(crate) plugin_directory: PathBuf,
}

impl ManagedMod {
    pub fn new(url: &str, game_directory: PathBuf, plugin_directory: PathBuf) -> Self {
        let file_type = url_parse_file_type(url);
        ManagedMod {
            url: url.to_string(),
            file_type,
            staging_location: game_directory.join("mods_staging"),
            installed: false,
            downloaded: false,
            game_directory,
            plugin_directory,
        }
    }

    /// Checks if the extracted mod is a BepInEx framework mod.
    fn is_bepinex(&self, extract_path: &Path) -> bool {
        debug!("Checking if mod is BepInEx framework...");
        for entry in WalkDir::new(extract_path).into_iter().flatten() {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            if file_name == "winhttp.dll" || file_name == "bepinex" {
                return true;
            }
        }
        false
    }

    pub fn download(&mut self) -> Result<(), ModError> {
        debug!("Initializing mod download...");
        if !self.staging_location.exists() {
            create_dir_all(&self.staging_location)
                .map_err(|e| ModError::DirectoryCreationError(e.to_string()))?;
        }

        let parsed_url = Url::parse(&self.url).map_err(|_| ModError::InvalidUrl)?;
        let mut response = reqwest::blocking::get(parsed_url)
            .map_err(|e| ModError::DownloadError(e.to_string()))?;

        if !SUPPORTED_FILE_TYPES.contains(&self.file_type.as_str()) {
            debug!("Updating redirect URL: {}", &self.url);
            self.url = response.url().to_string();
            self.file_type = url_parse_file_type(response.url().as_ref());
        }

        let file_name = parse_file_name(
            &Url::parse(&self.url).unwrap(),
            &format!("{}.{}", get_md5_hash(&self.url), &self.file_type),
        );
        self.staging_location = self.staging_location.join(file_name);
        debug!("Downloading to: {:?}", self.staging_location);

        let mut file = File::create(&self.staging_location)
            .map_err(|e| ModError::FileCreateError(e.to_string()))?;
        response
            .copy_to(&mut file)
            .map_err(|e| ModError::DownloadError(e.to_string()))?;
        self.downloaded = true;
        debug!("Download complete: {}", &self.url);
        Ok(())
    }

    pub fn install(&mut self) -> Result<(), ModError> {
        if self.staging_location.is_dir() {
            error!("Invalid install path: {:?}", self.staging_location);
            return Err(ModError::InvalidStagingLocation);
        }

        let temp_dir = tempdir().map_err(|e| ModError::TempDirCreationError(e.to_string()))?;
        debug!("Created temp directory: {:?}", temp_dir.path());

        {
            let zip_file = File::open(&self.staging_location)
                .map_err(|e| ModError::FileOpenError(e.to_string()))?;
            let mut archive =
                ZipArchive::new(zip_file).map_err(|e| ModError::ZipArchiveError(e.to_string()))?;
            archive
                .extract(temp_dir.path())
                .map_err(|e| ModError::ExtractionError(e.to_string()))?;
            normalize_paths(temp_dir.path())
                .map_err(|e| ModError::ExtractionError(e.to_string()))?;
        }

        let is_bepinex = self.is_bepinex(temp_dir.path());
        let final_dir = if is_bepinex {
            &self.game_directory
        } else {
            &self.plugin_directory
        };

        let options = CopyOptions {
            overwrite: true,
            skip_exist: false,
            buffer_size: 0,
            copy_inside: false,
            content_only: true,
            depth: 0,
        };

        create_dir_all(final_dir).map_err(|e| ModError::DirectoryCreationError(e.to_string()))?;
        dir::move_dir(temp_dir, final_dir, &options)
            .map_err(|e| ModError::FileMoveError(e.to_string()))?;

        self.installed = true;
        Ok(())
    }
}

impl TryFrom<String> for ManagedMod {
    type Error = ModError;

    fn try_from(url: String) -> Result<Self, Self::Error> {
        if is_valid_url(&url) {
            Ok(ManagedMod::new(&url, PathBuf::new(), PathBuf::new()))
        } else if let Some((author, mod_name, version)) = parse_mod_string(&url) {
            let constructed_url = format!(
                "https://gcdn.thunderstore.io/live/repository/packages/{}-{}-{}.zip",
                author, mod_name, version
            );
            Ok(ManagedMod::new(
                &constructed_url,
                PathBuf::new(),
                PathBuf::new(),
            ))
        } else {
            Err(ModError::InvalidUrl)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;
    use zip::write::{FileOptions, ZipWriter};

    // Helper function to create a dummy ZIP file.
    // If `include_bepinex` is true, it creates a file "winhttp.dll" inside the archive.
    // Otherwise, it creates a dummy file "dummy.txt".
    fn create_dummy_zip(file_path: &Path, include_bepinex: bool) -> std::io::Result<()> {
        let file = File::create(file_path)?;
        let mut zip = ZipWriter::new(file);
        // Explicitly annotate the type for FileOptions
        let options: FileOptions<()> = FileOptions::default();
        if include_bepinex {
            zip.start_file("winhttp.dll", options)?;
            zip.write_all(b"dummy dll content")?;
        } else {
            zip.start_file("dummy.txt", options)?;
            zip.write_all(b"dummy content")?;
        }
        zip.finish()?;
        Ok(())
    }

    #[test]
    fn test_install_non_bepinex_mod() {
        // Create temporary directories for game and plugin directories.
        let game_dir = tempdir().unwrap();
        let plugin_dir = tempdir().unwrap();

        // Create a staging directory and dummy ZIP file (without BepInEx indicator).
        let staging_dir = game_dir.path().join("mods_staging");
        fs::create_dir_all(&staging_dir).unwrap();
        let staging_file = staging_dir.join("dummy.zip");
        create_dummy_zip(&staging_file, false).unwrap();

        // Build the ManagedMod instance and override staging_location to our dummy ZIP.
        let mut mod_instance = ManagedMod {
            url: "http://example.com/dummy.zip".to_string(),
            file_type: "zip".to_string(),
            staging_location: staging_file,
            installed: false,
            downloaded: true,
            game_directory: game_dir.path().to_path_buf(),
            plugin_directory: plugin_dir.path().to_path_buf(),
        };

        mod_instance.install().unwrap();

        // Verify that the plugin directory contains the file "dummy.txt".
        let mut found = false;
        for entry in fs::read_dir(plugin_dir.path()).unwrap() {
            let entry = entry.unwrap();
            if entry.file_name().to_string_lossy().contains("dummy.txt") {
                found = true;
                break;
            }
        }
        assert!(found, "dummy.txt not found in plugin directory");
    }

    #[test]
    fn test_install_bepinex_mod() {
        // Create temporary directories for game and plugin directories.
        let game_dir = tempdir().unwrap();
        let plugin_dir = tempdir().unwrap();

        // Create a staging directory and dummy ZIP file that simulates a BepInEx mod.
        let staging_dir = game_dir.path().join("mods_staging");
        fs::create_dir_all(&staging_dir).unwrap();
        let staging_file = staging_dir.join("bepinex_dummy.zip");
        create_dummy_zip(&staging_file, true).unwrap();

        // Build the ManagedMod instance and override staging_location to our dummy ZIP.
        let mut mod_instance = ManagedMod {
            url: "http://example.com/bepinex_dummy.zip".to_string(),
            file_type: "zip".to_string(),
            staging_location: staging_file,
            installed: false,
            downloaded: true,
            game_directory: game_dir.path().to_path_buf(),
            plugin_directory: plugin_dir.path().to_path_buf(),
        };

        mod_instance.install().unwrap();

        // Verify that the game directory contains the file "winhttp.dll".
        let mut found = false;
        for entry in fs::read_dir(game_dir.path()).unwrap() {
            let entry = entry.unwrap();
            if entry.file_name().to_string_lossy().contains("winhttp.dll") {
                found = true;
                break;
            }
        }
        assert!(found, "winhttp.dll not found in game directory");
    }

    #[test]
    fn test_try_from_valid_url() {
        let mod_instance = ManagedMod::try_from("http://example.com/mod.zip".to_string()).unwrap();
        assert_eq!(mod_instance.url, "http://example.com/mod.zip");
    }

    #[test]
    fn test_try_from_invalid_url() {
        let result = ManagedMod::try_from("invalid_url".to_string());
        assert!(result.is_err());
    }
}
