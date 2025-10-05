use std::collections::HashMap;

use super::core::ConfigManager;
use crate::config::PluginConfig;
use crate::error::Result;
use crate::plugins::interface::PluginManifest;

impl ConfigManager {
    /// Register plugin manifest
    pub async fn register_plugin(&self, manifest: PluginManifest) -> Result<()> {
        let plugin_id = manifest.id.clone();

        // Store manifest
        {
            let mut manifests = self.plugin_manifests.write().await;
            manifests.insert(plugin_id.clone(), manifest.clone());
        }

        // Load or create configuration
        let config = match self.store.load_config(&plugin_id).await? {
            Some(existing) => existing,
            None => {
                // Create default configuration
                let mut default_config = HashMap::new();
                let mut default_preferences = HashMap::new();

                // Apply defaults from manifest
                for field in &manifest.configuration {
                    if let Some(default_value) = &field.default {
                        default_config.insert(field.name.clone(), default_value.clone());
                    }
                }

                for field in &manifest.preferences {
                    default_preferences.insert(field.key.clone(), field.default.clone());
                }

                let config = PluginConfig {
                    plugin_id: plugin_id.clone(),
                    version: manifest.version.clone(),
                    configuration: default_config,
                    preferences: default_preferences,
                    enabled: true, // Plugins are enabled by default
                    last_modified: chrono::Utc::now(),
                };

                // Save default configuration
                self.store.save_config(&config).await?;
                config
            },
        };

        // Store in memory
        {
            let mut loaded_configs = self.loaded_configs.write().await;
            loaded_configs.insert(plugin_id.clone(), config.clone());
        }

        Ok(())
    }
}
