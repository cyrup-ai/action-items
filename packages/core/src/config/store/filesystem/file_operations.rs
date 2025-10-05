//! File I/O operations and path management for filesystem config store

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::super::types::PluginConfig;
use super::serialization::ConfigSerializer;
use crate::error::{Error, Result};

/// File operations handler for configuration storage
pub struct FileOperations {
    base_path: PathBuf,
    serializer: ConfigSerializer,
}

impl FileOperations {
    pub fn new(base_path: PathBuf, serializer: ConfigSerializer) -> Self {
        Self {
            base_path,
            serializer,
        }
    }

    /// Get config file path for plugin
    pub fn config_path(&self, plugin_id: &str) -> PathBuf {
        let extension = self.serializer.file_extension();
        self.base_path
            .join("plugins")
            .join(format!("{plugin_id}.{extension}"))
    }

    /// Get change history file path
    pub fn changes_path(&self) -> PathBuf {
        self.base_path.join("changes.json")
    }

    /// Ensure directory exists
    pub async fn ensure_directory(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::ConfigurationError(format!(
                    "Failed to create directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
        Ok(())
    }

    /// Load configuration from file
    pub async fn load_config(&self, plugin_id: &str) -> Result<Option<PluginConfig>> {
        let path = self.config_path(plugin_id);

        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
            Error::ConfigurationError(format!(
                "Failed to read config file {}: {e}",
                path.display()
            ))
        })?;

        let config = self.serializer.deserialize_config(&content)?;
        Ok(Some(config))
    }

    /// Save configuration to file
    pub async fn save_config(&self, config: &PluginConfig) -> Result<()> {
        let path = self.config_path(&config.plugin_id);
        self.ensure_directory(&path).await?;

        let content = self.serializer.serialize_config(config)?;

        tokio::fs::write(&path, content).await.map_err(|e| {
            Error::ConfigurationError(format!(
                "Failed to write config file {}: {e}",
                path.display()
            ))
        })?;

        Ok(())
    }

    /// Load all configurations from the plugins directory
    pub async fn load_all_configs(&self) -> Result<HashMap<String, PluginConfig>> {
        let plugins_dir = self.base_path.join("plugins");

        if !plugins_dir.exists() {
            return Ok(HashMap::new());
        }

        let mut configs = HashMap::new();
        let mut entries = tokio::fs::read_dir(&plugins_dir).await.map_err(|e| {
            Error::ConfigurationError(format!("Failed to read plugins directory: {e}"))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            Error::ConfigurationError(format!("Failed to read directory entry: {e}"))
        })? {
            let path = entry.path();
            if path.is_file()
                && let Some(stem) = path.file_stem()
                && let Some(plugin_id) = stem.to_str()
                && let Some(config) = self.load_config(plugin_id).await?
            {
                configs.insert(plugin_id.to_string(), config);
            }
        }

        Ok(configs)
    }

    /// Delete configuration file
    pub async fn delete_config(&self, plugin_id: &str) -> Result<()> {
        let path = self.config_path(plugin_id);

        if path.exists() {
            tokio::fs::remove_file(&path).await.map_err(|e| {
                Error::ConfigurationError(format!(
                    "Failed to delete config file {}: {e}",
                    path.display()
                ))
            })?;
        }

        Ok(())
    }

    /// Check if configuration exists
    pub async fn exists(&self, plugin_id: &str) -> Result<bool> {
        let path = self.config_path(plugin_id);
        Ok(path.exists())
    }
}
