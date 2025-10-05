//! Change tracking functionality for configuration history

use super::super::types::ConfigChange;
use super::file_operations::FileOperations;
use crate::error::{Error, Result};

/// Change tracking operations for configuration history
pub struct ChangeTracker<'a> {
    file_ops: &'a FileOperations,
}

impl<'a> ChangeTracker<'a> {
    pub fn new(file_ops: &'a FileOperations) -> Self {
        Self { file_ops }
    }

    /// Track a configuration change
    pub async fn track_change(&self, change: ConfigChange) -> Result<()> {
        let path = self.file_ops.changes_path();
        self.file_ops.ensure_directory(&path).await?;

        // Load existing changes
        let mut changes = if path.exists() {
            let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
                Error::ConfigurationError(format!("Failed to read changes file: {e}"))
            })?;
            serde_json::from_str::<Vec<ConfigChange>>(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Add new change
        changes.push(change);

        // Keep only last 1000 changes for performance
        if changes.len() > 1000 {
            let start = changes.len() - 1000;
            changes.drain(..start);
        }

        // Save changes
        let content = serde_json::to_string_pretty(&changes)
            .map_err(|e| Error::ConfigurationError(format!("Failed to serialize changes: {e}")))?;

        tokio::fs::write(&path, content)
            .await
            .map_err(|e| Error::ConfigurationError(format!("Failed to write changes file: {e}")))?;

        Ok(())
    }

    /// Get change history for a specific plugin
    pub async fn get_change_history(
        &self,
        plugin_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ConfigChange>> {
        let path = self.file_ops.changes_path();

        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| Error::ConfigurationError(format!("Failed to read changes file: {e}")))?;

        let all_changes: Vec<ConfigChange> = serde_json::from_str(&content).unwrap_or_default();

        let mut plugin_changes: Vec<ConfigChange> = all_changes
            .into_iter()
            .filter(|change| change.plugin_id == plugin_id)
            .collect();

        // Sort by timestamp (newest first)
        plugin_changes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            plugin_changes.truncate(limit);
        }

        Ok(plugin_changes)
    }
}
