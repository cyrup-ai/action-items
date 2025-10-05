//! Core data structures for configuration storage

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Configuration value type - supports all JSON types
pub type ConfigValue = serde_json::Value;

/// Plugin configuration data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_id: String,
    pub version: String,
    pub configuration: HashMap<String, ConfigValue>,
    pub preferences: HashMap<String, ConfigValue>,
    pub enabled: bool,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

/// Configuration change tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub plugin_id: String,
    pub field_name: String,
    pub old_value: Option<ConfigValue>,
    pub new_value: ConfigValue,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub change_type: ChangeType,
}

/// Types of configuration changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Configuration,
    Preference,
    Enable,
    Disable,
}

/// Storage format options for configuration files
#[derive(Debug, Clone)]
pub enum StorageFormat {
    Json,
    JsonPretty,
    Toml,
}
