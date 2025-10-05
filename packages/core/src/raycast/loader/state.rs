//! Raycast sync state management

use std::fs;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{Error, Result};

const UPDATE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

#[derive(Debug, Serialize, Deserialize)]
pub struct RaycastSyncState {
    pub last_sync: SystemTime,
    pub last_commit: String,
    pub extension_count: usize,
}

impl RaycastSyncState {
    /// Check if we should update based on the 24-hour interval
    pub fn should_update(&self) -> bool {
        let elapsed = SystemTime::now()
            .duration_since(self.last_sync)
            .unwrap_or(Duration::MAX);
        elapsed > UPDATE_INTERVAL
    }

    /// Save the sync state to file
    pub fn save_to_file(&self, state_file: &std::path::Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            Error::SerializationError(format!("Failed to serialize sync state: {e}"))
        })?;

        fs::write(state_file, json)
            .map_err(|e| Error::IoError(format!("Failed to write sync state: {e}")))?;

        info!(
            extension_count = self.extension_count,
            last_commit = %self.last_commit,
            "Raycast extensions sync completed"
        );
        Ok(())
    }

    /// Load the sync state from file
    pub fn load_from_file(state_file: &std::path::Path) -> Result<Self> {
        let content = fs::read_to_string(state_file)
            .map_err(|e| Error::IoError(format!("Failed to read sync state: {e}")))?;

        serde_json::from_str(&content)
            .map_err(|e| Error::SerializationError(format!("Failed to parse sync state: {e}")))
    }
}
