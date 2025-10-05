use std::collections::HashMap;

use bevy::prelude::*;

use crate::config::{ConfigValue, PluginConfig, ValidationResult};

/// Configuration management events
#[derive(Event, Debug, Clone)]
pub enum ConfigEvent {
    /// Plugin configuration loaded
    ConfigLoaded {
        plugin_id: String,
        config: PluginConfig,
    },
    /// Configuration value changed
    ConfigChanged {
        plugin_id: String,
        field_name: String,
        old_value: Option<ConfigValue>,
        new_value: ConfigValue,
    },
    /// Plugin enabled/disabled
    PluginToggled { plugin_id: String, enabled: bool },
    /// Validation completed
    ValidationCompleted {
        plugin_id: String,
        results: HashMap<String, ValidationResult>,
        is_valid: bool,
    },
    /// Configuration saved
    ConfigSaved { plugin_id: String },
    /// Configuration error
    ConfigError { plugin_id: String, error: String },
}
