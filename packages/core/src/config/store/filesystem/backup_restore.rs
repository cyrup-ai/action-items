//! Backup and restore functionality for configuration data

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::types::{ConfigChange, PluginConfig};
use super::file_operations::FileOperations;
use crate::error::{Error, Result};

/// Backup and restore operations for configuration data
pub struct BackupRestore<'a> {
    file_ops: &'a FileOperations,
}

impl<'a> BackupRestore<'a> {
    pub fn new(file_ops: &'a FileOperations) -> Self {
        Self { file_ops }
    }

    /// Create a backup of all configuration data
    pub async fn backup(&self, backup_path: &Path) -> Result<()> {
        let configs = self.file_ops.load_all_configs().await?;
        let changes = if self.file_ops.changes_path().exists() {
            let content = tokio::fs::read_to_string(self.file_ops.changes_path())
                .await
                .map_err(|e| Error::ConfigurationError(format!("Failed to read changes: {e}")))?;
            serde_json::from_str::<Vec<ConfigChange>>(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        #[derive(Serialize)]
        struct Backup {
            configs: HashMap<String, PluginConfig>,
            changes: Vec<ConfigChange>,
            backup_timestamp: chrono::DateTime<chrono::Utc>,
        }

        let backup = Backup {
            configs,
            changes,
            backup_timestamp: chrono::Utc::now(),
        };

        self.file_ops.ensure_directory(backup_path).await?;
        let content = serde_json::to_string_pretty(&backup)
            .map_err(|e| Error::ConfigurationError(format!("Failed to serialize backup: {e}")))?;

        tokio::fs::write(backup_path, content)
            .await
            .map_err(|e| Error::ConfigurationError(format!("Failed to write backup file: {e}")))?;

        Ok(())
    }

    /// Restore configuration data from a backup
    pub async fn restore(&self, backup_path: &Path) -> Result<()> {
        #[derive(Deserialize)]
        struct Backup {
            configs: HashMap<String, PluginConfig>,
            changes: Vec<ConfigChange>,
        }

        let content = tokio::fs::read_to_string(backup_path)
            .await
            .map_err(|e| Error::ConfigurationError(format!("Failed to read backup file: {e}")))?;

        let backup: Backup = serde_json::from_str(&content)
            .map_err(|e| Error::ConfigurationError(format!("Failed to deserialize backup: {e}")))?;

        // Restore all configurations
        for (_, config) in backup.configs {
            self.file_ops.save_config(&config).await?;
        }

        // Restore changes
        let changes_content = serde_json::to_string_pretty(&backup.changes)
            .map_err(|e| Error::ConfigurationError(format!("Failed to serialize changes: {e}")))?;

        let changes_path = self.file_ops.changes_path();
        self.file_ops.ensure_directory(&changes_path).await?;
        tokio::fs::write(&changes_path, changes_content)
            .await
            .map_err(|e| Error::ConfigurationError(format!("Failed to write changes file: {e}")))?;

        Ok(())
    }
}
