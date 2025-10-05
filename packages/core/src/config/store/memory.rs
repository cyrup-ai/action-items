//! In-memory configuration store implementation for testing

use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;

use super::trait_definition::ConfigStore;
use super::types::{ConfigChange, PluginConfig};
use crate::error::Result;

/// In-memory configuration store for testing
#[derive(Default)]
pub struct MemoryConfigStore {
    configs: tokio::sync::RwLock<HashMap<String, PluginConfig>>,
    changes: tokio::sync::RwLock<Vec<ConfigChange>>,
}

#[async_trait]
impl ConfigStore for MemoryConfigStore {
    async fn load_config(&self, plugin_id: &str) -> Result<Option<PluginConfig>> {
        let configs = self.configs.read().await;
        Ok(configs.get(plugin_id).cloned())
    }

    async fn save_config(&self, config: &PluginConfig) -> Result<()> {
        let mut configs = self.configs.write().await;
        configs.insert(config.plugin_id.clone(), config.clone());
        Ok(())
    }

    async fn load_all_configs(&self) -> Result<HashMap<String, PluginConfig>> {
        let configs = self.configs.read().await;
        Ok(configs.clone())
    }

    async fn delete_config(&self, plugin_id: &str) -> Result<()> {
        let mut configs = self.configs.write().await;
        configs.remove(plugin_id);
        Ok(())
    }

    async fn exists(&self, plugin_id: &str) -> Result<bool> {
        let configs = self.configs.read().await;
        Ok(configs.contains_key(plugin_id))
    }

    async fn track_change(&self, change: ConfigChange) -> Result<()> {
        let mut changes = self.changes.write().await;
        changes.push(change);
        Ok(())
    }

    async fn get_change_history(
        &self,
        plugin_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ConfigChange>> {
        let changes = self.changes.read().await;
        let mut plugin_changes: Vec<ConfigChange> = changes
            .iter()
            .filter(|change| change.plugin_id == plugin_id)
            .cloned()
            .collect();

        plugin_changes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            plugin_changes.truncate(limit);
        }

        Ok(plugin_changes)
    }

    async fn backup(&self, _backup_path: &Path) -> Result<()> {
        // Memory store doesn't need backup
        Ok(())
    }

    async fn restore(&self, _backup_path: &Path) -> Result<()> {
        // Memory store doesn't need restore
        Ok(())
    }
}
