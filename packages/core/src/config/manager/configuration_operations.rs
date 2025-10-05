use bevy::prelude::*;

use super::core::ConfigManager;
use super::events::ConfigEvent;
use crate::config::{ChangeType, ConfigChange, ConfigValue, ValidationResult};
use crate::error::{Error, Result};

impl ConfigManager {
    /// Update configuration field
    pub async fn update_config_field(
        &self,
        plugin_id: &str,
        field_name: &str,
        new_value: ConfigValue,
        event_writer: &mut EventWriter<'_, ConfigEvent>,
    ) -> Result<ValidationResult> {
        // Get current configuration
        let mut config = {
            let loaded_configs = self.loaded_configs.read().await;
            loaded_configs
                .get(plugin_id)
                .ok_or_else(|| {
                    Error::ConfigurationError(format!("Plugin '{plugin_id}' not found"))
                })?
                .clone()
        };

        let old_value = config.configuration.get(field_name).cloned();

        // Get field definition for validation
        let field = {
            let manifests = self.plugin_manifests.read().await;
            let manifest = manifests.get(plugin_id).ok_or_else(|| {
                Error::ConfigurationError(format!("Manifest for plugin '{plugin_id}' not found"))
            })?;

            manifest
                .configuration
                .iter()
                .find(|f| f.name == field_name)
                .ok_or_else(|| {
                    Error::ConfigurationError(format!("Field '{field_name}' not found in manifest"))
                })?
                .clone()
        };

        // Validate new value
        let validation_result = {
            let mut validation_engine = self.validation_engine.write().await;
            validation_engine.validate_field(&field, &Some(new_value.clone()))
        };

        // Send validation event
        event_writer.write(ConfigEvent::ValidationCompleted {
            plugin_id: plugin_id.to_string(),
            results: [(field_name.to_string(), validation_result.clone())].into(),
            is_valid: validation_result.is_valid,
        });

        if !validation_result.is_valid {
            return Ok(validation_result);
        }

        // Update configuration
        config
            .configuration
            .insert(field_name.to_string(), new_value.clone());
        config.last_modified = chrono::Utc::now();

        // Track change
        let change = ConfigChange {
            plugin_id: plugin_id.to_string(),
            field_name: field_name.to_string(),
            old_value: old_value.clone(),
            new_value: new_value.clone(),
            timestamp: chrono::Utc::now(),
            change_type: ChangeType::Configuration,
        };

        self.store.track_change(change).await?;

        // Update in memory
        {
            let mut loaded_configs = self.loaded_configs.write().await;
            loaded_configs.insert(plugin_id.to_string(), config.clone());
        }

        // Send change event
        event_writer.write(ConfigEvent::ConfigChanged {
            plugin_id: plugin_id.to_string(),
            field_name: field_name.to_string(),
            old_value,
            new_value,
        });

        // Auto-save if enabled
        if self.auto_save {
            self.save_config(plugin_id, event_writer).await?;
        } else {
            // Store as pending change
            let mut pending = self.pending_changes.write().await;
            let plugin_changes = pending.entry(plugin_id.to_string()).or_default();
            plugin_changes.insert(
                field_name.to_string(),
                config.configuration[field_name].clone(),
            );
        }

        Ok(validation_result)
    }

    /// Update preference field
    pub async fn update_preference_field(
        &self,
        plugin_id: &str,
        field_name: &str,
        new_value: ConfigValue,
        event_writer: &mut EventWriter<'_, ConfigEvent>,
    ) -> Result<()> {
        // Get current configuration
        let mut config = {
            let loaded_configs = self.loaded_configs.read().await;
            loaded_configs
                .get(plugin_id)
                .ok_or_else(|| {
                    Error::ConfigurationError(format!("Plugin '{plugin_id}' not found"))
                })?
                .clone()
        };

        let old_value = config.preferences.get(field_name).cloned();

        // Update preference
        config
            .preferences
            .insert(field_name.to_string(), new_value.clone());
        config.last_modified = chrono::Utc::now();

        // Track change
        let change = ConfigChange {
            plugin_id: plugin_id.to_string(),
            field_name: field_name.to_string(),
            old_value: old_value.clone(),
            new_value: new_value.clone(),
            timestamp: chrono::Utc::now(),
            change_type: ChangeType::Preference,
        };

        self.store.track_change(change).await?;

        // Update in memory
        {
            let mut loaded_configs = self.loaded_configs.write().await;
            loaded_configs.insert(plugin_id.to_string(), config.clone());
        }

        // Send change event
        event_writer.write(ConfigEvent::ConfigChanged {
            plugin_id: plugin_id.to_string(),
            field_name: field_name.to_string(),
            old_value,
            new_value,
        });

        // Auto-save if enabled
        if self.auto_save {
            self.save_config(plugin_id, event_writer).await?;
        }

        Ok(())
    }

    /// Toggle plugin enabled state
    pub async fn toggle_plugin(
        &self,
        plugin_id: &str,
        enabled: bool,
        event_writer: &mut EventWriter<'_, ConfigEvent>,
    ) -> Result<()> {
        // Get current configuration
        let mut config = {
            let loaded_configs = self.loaded_configs.read().await;
            loaded_configs
                .get(plugin_id)
                .ok_or_else(|| {
                    Error::ConfigurationError(format!("Plugin '{plugin_id}' not found"))
                })?
                .clone()
        };

        let old_enabled = config.enabled;
        config.enabled = enabled;
        config.last_modified = chrono::Utc::now();

        // Track change
        let change = ConfigChange {
            plugin_id: plugin_id.to_string(),
            field_name: "enabled".to_string(),
            old_value: Some(serde_json::Value::Bool(old_enabled)),
            new_value: serde_json::Value::Bool(enabled),
            timestamp: chrono::Utc::now(),
            change_type: if enabled {
                ChangeType::Enable
            } else {
                ChangeType::Disable
            },
        };

        self.store.track_change(change).await?;

        // Update in memory
        {
            let mut loaded_configs = self.loaded_configs.write().await;
            loaded_configs.insert(plugin_id.to_string(), config.clone());
        }

        // Send toggle event
        event_writer.write(ConfigEvent::PluginToggled {
            plugin_id: plugin_id.to_string(),
            enabled,
        });

        // Auto-save if enabled
        if self.auto_save {
            self.save_config(plugin_id, event_writer).await?;
        }

        Ok(())
    }
}
