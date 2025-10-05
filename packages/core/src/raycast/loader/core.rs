//! Main Raycast loader implementation

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use tracing::info;

use super::extension::RaycastExtension;
use super::state::RaycastSyncState;
use crate::error::{Error, Result};

const RAYCAST_EXTENSIONS_REPO: &str = "https://github.com/raycast/extensions.git";

pub struct RaycastLoader {
    base_path: PathBuf,
    extensions_path: PathBuf,
    state_file: PathBuf,
}

impl RaycastLoader {
    pub fn new(config_dir: &Path) -> Self {
        let base_path = config_dir.join("raycast");
        let extensions_path = base_path.join("extensions");
        let state_file = base_path.join("sync_state.json");

        Self {
            base_path,
            extensions_path,
            state_file,
        }
    }

    /// Initialize the Raycast extensions directory and clone if needed
    pub fn initialize(&self) -> Result<()> {
        // Create base directory
        fs::create_dir_all(&self.base_path)
            .map_err(|e| Error::IoError(format!("Failed to create Raycast directory: {e}")))?;

        // Check if we need to clone or update
        if !self.extensions_path.exists() {
            self.clone_extensions()?;
        } else if self.should_update()? {
            self.update_extensions()?;
        }

        Ok(())
    }

    /// Check if we should update based on the 24-hour interval
    fn should_update(&self) -> Result<bool> {
        if let Ok(state) = RaycastSyncState::load_from_file(&self.state_file) {
            Ok(state.should_update())
        } else {
            // No state file, should update
            Ok(true)
        }
    }

    /// Clone the Raycast extensions repository
    fn clone_extensions(&self) -> Result<()> {
        info!(
            repository = RAYCAST_EXTENSIONS_REPO,
            target_path = %self.extensions_path.display(),
            "Cloning Raycast extensions repository"
        );

        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1", // Shallow clone for speed
                "--single-branch",
                "--branch",
                "main",
                RAYCAST_EXTENSIONS_REPO,
                self.extensions_path.to_str().ok_or_else(|| {
                    Error::SystemError("Invalid UTF-8 in extensions path".to_string())
                })?,
            ])
            .output()
            .map_err(|e| Error::SystemError(format!("Failed to run git clone: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::SystemError(format!("Git clone failed: {stderr}")));
        }

        self.save_sync_state()?;
        Ok(())
    }

    /// Update the Raycast extensions repository
    fn update_extensions(&self) -> Result<()> {
        info!(
            repository = RAYCAST_EXTENSIONS_REPO,
            extensions_path = %self.extensions_path.display(),
            "Updating Raycast extensions"
        );

        // Git pull
        let output = Command::new("git")
            .args(["pull", "--rebase"])
            .current_dir(&self.extensions_path)
            .output()
            .map_err(|e| Error::SystemError(format!("Failed to run git pull: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::SystemError(format!("Git pull failed: {stderr}")));
        }

        self.save_sync_state()?;
        Ok(())
    }

    /// Get the current git commit hash
    fn get_current_commit(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.extensions_path)
            .output()
            .map_err(|e| Error::SystemError(format!("Failed to get git commit: {e}")))?;

        if !output.status.success() {
            return Err(Error::SystemError(
                "Failed to get current commit".to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Count the number of extensions
    fn count_extensions(&self) -> Result<usize> {
        let mut count = 0;
        let extensions_dir = self.extensions_path.join("extensions");

        if extensions_dir.exists() {
            for entry in (fs::read_dir(&extensions_dir)
                .map_err(|e| Error::IoError(format!("Failed to read extensions directory: {e}")))?)
            .flatten()
            {
                if entry.path().join("package.json").exists() {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Save the sync state
    fn save_sync_state(&self) -> Result<()> {
        let state = RaycastSyncState {
            last_sync: SystemTime::now(),
            last_commit: self.get_current_commit()?,
            extension_count: self.count_extensions()?,
        };

        state.save_to_file(&self.state_file)
    }

    /// Get the path to a specific extension
    pub fn get_extension_path(&self, extension_id: &str) -> PathBuf {
        self.extensions_path.join("extensions").join(extension_id)
    }

    /// List all available Raycast extensions
    pub fn list_extensions(&self) -> Result<Vec<RaycastExtension>> {
        let mut extensions = Vec::new();
        let extensions_dir = self.extensions_path.join("extensions");

        if !extensions_dir.exists() {
            return Ok(extensions);
        }

        for entry in fs::read_dir(&extensions_dir)
            .map_err(|e| Error::IoError(format!("Failed to read extensions directory: {e}")))?
        {
            let entry = entry
                .map_err(|e| Error::IoError(format!("Failed to read directory entry: {e}")))?;
            let path = entry.path();

            if path.is_dir()
                && let Ok(extension) = RaycastExtension::from_path(&path)
            {
                extensions.push(extension);
            }
        }

        Ok(extensions)
    }
}
