//! Main FileSystemConfigStore implementation

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::super::trait_definition::ConfigStore;
use super::super::types::{ConfigChange, PluginConfig, StorageFormat};
use super::backup_restore::BackupRestore;
use super::change_tracking::ChangeTracker;
use super::file_operations::FileOperations;
use super::serialization::ConfigSerializer;
use crate::error::Result;

/// File system-based configuration store
pub struct FileSystemConfigStore {
    file_ops: FileOperations,
}

impl FileSystemConfigStore {
    /// Create new file system config store
    pub fn new<P: Into<PathBuf>>(base_path: P, format: StorageFormat) -> Self {
        let base_path = base_path.into();
        let serializer = ConfigSerializer::new(format);
        let file_ops = FileOperations::new(base_path, serializer);

        Self { file_ops }
    }
}

#[async_trait]
impl ConfigStore for FileSystemConfigStore {
    async fn load_config(&self, plugin_id: &str) -> Result<Option<PluginConfig>> {
        self.file_ops.load_config(plugin_id).await
    }

    async fn save_config(&self, config: &PluginConfig) -> Result<()> {
        self.file_ops.save_config(config).await
    }

    async fn load_all_configs(&self) -> Result<HashMap<String, PluginConfig>> {
        self.file_ops.load_all_configs().await
    }

    async fn delete_config(&self, plugin_id: &str) -> Result<()> {
        self.file_ops.delete_config(plugin_id).await
    }

    async fn exists(&self, plugin_id: &str) -> Result<bool> {
        self.file_ops.exists(plugin_id).await
    }

    async fn track_change(&self, change: ConfigChange) -> Result<()> {
        let tracker = ChangeTracker::new(&self.file_ops);
        tracker.track_change(change).await
    }

    async fn get_change_history(
        &self,
        plugin_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ConfigChange>> {
        let tracker = ChangeTracker::new(&self.file_ops);
        tracker.get_change_history(plugin_id, limit).await
    }

    async fn backup(&self, backup_path: &Path) -> Result<()> {
        let backup_restore = BackupRestore::new(&self.file_ops);
        backup_restore.backup(backup_path).await
    }

    async fn restore(&self, backup_path: &Path) -> Result<()> {
        let backup_restore = BackupRestore::new(&self.file_ops);
        backup_restore.restore(backup_path).await
    }
}
