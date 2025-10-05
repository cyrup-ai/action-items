//! XDG Base Directory Specification compliant directory management
//!
//! Provides centralized directory management following the XDG Base Directory Specification
//! for consistent file organization across user environments.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Centralized directory management following XDG Base Directory Specification
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct AppDirectories {
    config_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    state_dir: PathBuf,
}

impl AppDirectories {
    /// Create new XDG-compliant directory structure
    pub fn new() -> Self {
        let base_name = "action-items";

        Self {
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| std::env::temp_dir().join("action-items-config"))
                .join(base_name),
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| std::env::temp_dir().join("action-items-data"))
                .join(base_name),
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| std::env::temp_dir().join("action-items-cache"))
                .join(base_name),
            state_dir: dirs::state_dir()
                .unwrap_or_else(|| std::env::temp_dir().join("action-items-state"))
                .join(base_name),
        }
    }

    /// Get logs directory (XDG_DATA_HOME/action-items/logs)
    /// Used for all application log files
    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    /// Get logs archive directory (XDG_DATA_HOME/action-items/logs/archive)
    /// Used for compressed/archived log files
    pub fn logs_archive_dir(&self) -> PathBuf {
        self.logs_dir().join("archive")
    }

    /// Get test logs directory (XDG_DATA_HOME/action-items/logs/test)
    /// Used for test log files in testing configurations
    pub fn test_logs_dir(&self) -> PathBuf {
        self.logs_dir().join("test")
    }

    /// Get plugin data directory (XDG_DATA_HOME/action-items/plugins)
    /// Used for persistent plugin storage and data files
    pub fn plugin_data(&self) -> PathBuf {
        self.data_dir.join("plugins")
    }

    /// Get plugin cache directory (XDG_CACHE_HOME/action-items/plugins)
    /// Used for non-essential cached plugin data
    pub fn plugin_cache(&self) -> PathBuf {
        self.cache_dir.join("plugins")
    }

    /// Get plugin state directory (XDG_STATE_HOME/action-items/plugins)
    /// Used for plugin runtime state and activity data
    pub fn plugin_state(&self) -> PathBuf {
        self.state_dir.join("plugins")
    }

    /// Get main configuration directory (XDG_CONFIG_HOME/action-items)
    /// Used for application and plugin configuration files
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Get main data directory (XDG_DATA_HOME/action-items)
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Get main cache directory (XDG_CACHE_HOME/action-items)
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Get main state directory (XDG_STATE_HOME/action-items)
    pub fn state_dir(&self) -> &PathBuf {
        &self.state_dir
    }

    /// Create all necessary directories with proper permissions
    pub fn ensure_directories_exist(&self) -> Result<(), std::io::Error> {
        let directories = [
            &self.config_dir,
            &self.data_dir,
            &self.cache_dir,
            &self.state_dir,
            &self.logs_dir(),
            &self.logs_archive_dir(),
            &self.test_logs_dir(),
            &self.plugin_data(),
            &self.plugin_cache(),
            &self.plugin_state(),
        ];

        for dir in directories {
            std::fs::create_dir_all(dir)?;

            // Set proper permissions on Unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = std::fs::metadata(dir)?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755);
                std::fs::set_permissions(dir, permissions)?;
            }

            // Verify directory is writable with process-safe temp file
            let test_file = dir.join(format!(
                ".write_test_{}_{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ));

            // Retry logic for concurrent access scenarios
            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 5;
            let mut last_error = None;

            loop {
                attempts += 1;
                match std::fs::write(&test_file, b"test") {
                    Ok(()) => {
                        // Successfully wrote, now try to remove
                        if let Err(e) = std::fs::remove_file(&test_file) {
                            last_error = Some(e);
                            if attempts < MAX_ATTEMPTS {
                                std::thread::sleep(std::time::Duration::from_millis(
                                    10 * attempts as u64,
                                ));
                                continue;
                            }
                        }
                        break; // Success
                    },
                    Err(e) => {
                        last_error = Some(e);
                        if attempts >= MAX_ATTEMPTS {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempts as u64));
                    },
                }
            }

            if let Some(e) = last_error
                && attempts >= MAX_ATTEMPTS
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "Directory not writable after {} attempts: {}",
                        MAX_ATTEMPTS, e
                    ),
                ));
            }

            // Clean up any orphaned test files from previous runs
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str()
                        && name.starts_with(".write_test_")
                    {
                        // Safely get the current test file name for comparison
                        let current_test_name = test_file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown_test_file");

                        if name != current_test_name {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for AppDirectories {
    fn default() -> Self {
        Self::new()
    }
}
