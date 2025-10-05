use std::collections::HashMap;

use bevy::prelude::*;

use super::core::ConfigManager;
use super::events::ConfigEvent;
use crate::config::ValidationResult;
use crate::error::{Error, Result};

impl ConfigManager {
    /// Validate all configuration fields for a plugin
    pub async fn validate_plugin_config(
        &self,
        plugin_id: &str,
        event_writer: &mut EventWriter<'_, ConfigEvent>,
    ) -> Result<HashMap<String, ValidationResult>> {
        // Get manifest and config
        let (manifest, config) = {
            let manifests = self.plugin_manifests.read().await;
            let loaded_configs = self.loaded_configs.read().await;

            let manifest = manifests
                .get(plugin_id)
                .ok_or_else(|| {
                    Error::ConfigurationError(format!(
                        "Manifest for plugin '{plugin_id}' not found"
                    ))
                })?
                .clone();

            let config = loaded_configs
                .get(plugin_id)
                .ok_or_else(|| {
                    Error::ConfigurationError(format!("Config for plugin '{plugin_id}' not found"))
                })?
                .clone();

            (manifest, config)
        };

        // Validate all fields
        let results = {
            let mut validation_engine = self.validation_engine.write().await;
            validation_engine.validate_configuration(&manifest.configuration, &config.configuration)
        };

        let is_valid = results.values().all(|result| result.is_valid);

        // Send validation event
        event_writer.write(ConfigEvent::ValidationCompleted {
            plugin_id: plugin_id.to_string(),
            results: results.clone(),
            is_valid,
        });

        Ok(results)
    }
}
