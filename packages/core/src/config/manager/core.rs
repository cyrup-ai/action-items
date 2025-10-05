use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::Resource;
use tokio::sync::RwLock;

use crate::config::{ConfigStore, ConfigValue, PluginConfig, ValidationEngine};
use crate::plugins::interface::PluginManifest;

/// Main configuration manager
#[derive(Resource)]
pub struct ConfigManager {
    pub(super) store: Arc<dyn ConfigStore>,
    pub(super) validation_engine: Arc<RwLock<ValidationEngine>>,
    pub(super) loaded_configs: RwLock<HashMap<String, PluginConfig>>,
    pub(super) plugin_manifests: RwLock<HashMap<String, PluginManifest>>,
    pub(super) pending_changes: RwLock<HashMap<String, HashMap<String, ConfigValue>>>,
    pub(super) auto_save: bool,
}

impl ConfigManager {
    /// Create new configuration manager
    pub fn new(store: Arc<dyn ConfigStore>, auto_save: bool) -> Self {
        Self {
            store,
            validation_engine: Arc::new(RwLock::new(ValidationEngine::new())),
            loaded_configs: RwLock::new(HashMap::new()),
            plugin_manifests: RwLock::new(HashMap::new()),
            pending_changes: RwLock::new(HashMap::new()),
            auto_save,
        }
    }

    /// Get plugin configuration
    pub async fn get_config(&self, plugin_id: &str) -> Option<PluginConfig> {
        let loaded_configs = self.loaded_configs.read().await;
        loaded_configs.get(plugin_id).cloned()
    }

    /// Get all loaded configurations
    pub async fn get_all_configs(&self) -> HashMap<String, PluginConfig> {
        let loaded_configs = self.loaded_configs.read().await;
        loaded_configs.clone()
    }
}
