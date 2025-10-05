//! Format-specific serialization and deserialization for configuration files

// Note: Deserialize and Serialize will be used when serialization is implemented

use super::super::types::{PluginConfig, StorageFormat};
use crate::error::{Error, Result};

/// Serialization utilities for different storage formats
pub struct ConfigSerializer {
    format: StorageFormat,
}

impl ConfigSerializer {
    pub fn new(format: StorageFormat) -> Self {
        Self { format }
    }

    /// Serialize configuration based on format
    pub fn serialize_config(&self, config: &PluginConfig) -> Result<String> {
        match self.format {
            StorageFormat::Json => serde_json::to_string(config)
                .map_err(|e| Error::ConfigurationError(format!("JSON serialization failed: {e}"))),
            StorageFormat::JsonPretty => serde_json::to_string_pretty(config).map_err(|e| {
                Error::ConfigurationError(format!("JSON pretty serialization failed: {e}"))
            }),
            StorageFormat::Toml => toml::to_string(config)
                .map_err(|e| Error::ConfigurationError(format!("TOML serialization failed: {e}"))),
        }
    }

    /// Deserialize configuration based on format
    pub fn deserialize_config(&self, content: &str) -> Result<PluginConfig> {
        match self.format {
            StorageFormat::Json | StorageFormat::JsonPretty => serde_json::from_str(content)
                .map_err(|e| {
                    Error::ConfigurationError(format!("JSON deserialization failed: {e}"))
                }),
            StorageFormat::Toml => toml::from_str(content).map_err(|e| {
                Error::ConfigurationError(format!("TOML deserialization failed: {e}"))
            }),
        }
    }

    /// Get file extension for the current format
    pub fn file_extension(&self) -> &'static str {
        match self.format {
            StorageFormat::Json | StorageFormat::JsonPretty => "json",
            StorageFormat::Toml => "toml",
        }
    }
}
