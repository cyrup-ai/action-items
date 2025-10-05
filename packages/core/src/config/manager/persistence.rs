use std::collections::HashMap;

use bevy::prelude::*;
use log::warn;

use super::core::ConfigManager;
use super::events::ConfigEvent;
use crate::config::{ConfigChange, ConfigValue};
use crate::error::{Error, Result};

impl ConfigManager {
    /// Save configuration to persistent store
    pub async fn save_config(
        &self,
        plugin_id: &str,
        event_writer: &mut EventWriter<'_, ConfigEvent>,
    ) -> Result<()> {
        let config = {
            let loaded_configs = self.loaded_configs.read().await;
            loaded_configs
                .get(plugin_id)
                .ok_or_else(|| {
                    Error::ConfigurationError(format!("Plugin '{plugin_id}' not found"))
                })?
                .clone()
        };

        match self.store.save_config(&config).await {
            Ok(_) => {
                // Clear pending changes
                {
                    let mut pending = self.pending_changes.write().await;
                    pending.remove(plugin_id);
                }

                event_writer.write(ConfigEvent::ConfigSaved {
                    plugin_id: plugin_id.to_string(),
                });

                Ok(())
            },
            Err(e) => {
                event_writer.write(ConfigEvent::ConfigError {
                    plugin_id: plugin_id.to_string(),
                    error: e.to_string(),
                });

                Err(e)
            },
        }
    }

    /// Save all pending configurations
    pub async fn save_all_pending(
        &self,
        event_writer: &mut EventWriter<'_, ConfigEvent>,
    ) -> Result<()> {
        let pending_plugins: Vec<String> = {
            let pending = self.pending_changes.read().await;
            pending.keys().cloned().collect()
        };

        for plugin_id in pending_plugins {
            if let Err(e) = self.save_config(&plugin_id, event_writer).await {
                warn!("Failed to save config for plugin '{plugin_id}': {e}");
            }
        }

        Ok(())
    }

    /// Get pending changes for a plugin
    pub async fn get_pending_changes(&self, plugin_id: &str) -> HashMap<String, ConfigValue> {
        let pending = self.pending_changes.read().await;
        pending.get(plugin_id).cloned().unwrap_or_default()
    }

    /// Check if plugin has pending changes
    pub async fn has_pending_changes(&self, plugin_id: &str) -> bool {
        let pending = self.pending_changes.read().await;
        pending.contains_key(plugin_id)
    }

    /// Get configuration change history
    pub async fn get_change_history(
        &self,
        plugin_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ConfigChange>> {
        self.store.get_change_history(plugin_id, limit).await
    }
}
