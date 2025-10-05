//! Configuration store trait definition

use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;

use super::types::{ConfigChange, PluginConfig};
use crate::error::Result;

/// Trait for configuration persistence backends
#[async_trait]
pub trait ConfigStore: Send + Sync {
    /// Load plugin configuration
    async fn load_config(&self, plugin_id: &str) -> Result<Option<PluginConfig>>;

    /// Save plugin configuration
    async fn save_config(&self, config: &PluginConfig) -> Result<()>;

    /// Load all plugin configurations
    async fn load_all_configs(&self) -> Result<HashMap<String, PluginConfig>>;

    /// Delete plugin configuration
    async fn delete_config(&self, plugin_id: &str) -> Result<()>;

    /// Check if configuration exists
    async fn exists(&self, plugin_id: &str) -> Result<bool>;

    /// Track configuration change
    async fn track_change(&self, change: ConfigChange) -> Result<()>;

    /// Get configuration change history
    async fn get_change_history(
        &self,
        plugin_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ConfigChange>>;

    /// Backup all configurations
    async fn backup(&self, backup_path: &Path) -> Result<()>;

    /// Restore configurations from backup
    async fn restore(&self, backup_path: &Path) -> Result<()>;
}
