//! Runtime resource management for Deno plugins

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use tokio::sync::mpsc;

use crate::error::Result;
use crate::plugins::core::PluginMetadata;
use crate::plugins::services::PluginId;
use crate::runtime::deno::plugin_manager::PluginManager;
use crate::runtime::deno::types::RuntimeConfig;
use crate::runtime::plugin_wrapper::core::DenoPluginWrapper;
use crate::runtime::plugin_wrapper::request_handling::{
    ActionItemRequest, ActionItemResponse, RequestHandler, SearchMessage, StorageMessage,
};

/// Resource to manage the global Deno runtime
#[derive(Resource)]
pub struct DenoRuntimeResource {
    /// Plugin manager for loading/unloading plugins
    plugin_manager: PluginManager,
    /// Request handler for ActionItem operations
    request_handler: RequestHandler,
    /// Loaded plugin wrappers
    loaded_plugins: HashMap<PluginId, DenoPluginWrapper>,
    /// Runtime statistics
    stats: RuntimeStats,
}

/// Runtime statistics tracking
#[derive(Default)]
pub struct RuntimeStats {
    pub plugins_loaded: usize,
    pub plugins_failed: usize,
    pub requests_processed: u64,
    pub errors_encountered: u64,
    pub uptime_seconds: u64,
}

impl DenoRuntimeResource {
    /// Create a new Deno runtime resource
    pub fn new(
        storage_tx: mpsc::Sender<StorageMessage>,
        search_tx: mpsc::Sender<SearchMessage>,
    ) -> Result<Self> {
        let config = RuntimeConfig::default();
        let plugin_manager = PluginManager::new(config);

        // Create request handler with proper async channels
        let request_handler = RequestHandler::new(storage_tx.clone(), search_tx.clone());

        // Initialize the ActionItem ops handler
        let _ = crate::runtime::deno::ops::initialize_action_item_handler(storage_tx, search_tx);

        Ok(Self {
            plugin_manager,
            request_handler,
            loaded_plugins: HashMap::new(),
            stats: RuntimeStats::default(),
        })
    }

    /// Load a plugin from a file path
    pub async fn load_plugin(&mut self, plugin_path: &Path) -> Result<DenoPluginWrapper> {
        let plugin_id = self
            .plugin_manager
            .load_plugin(plugin_path)
            .map_err(crate::Error::PluginLoadError)?;

        // Get plugin metadata by loading manifest
        let manifest = self
            .plugin_manager
            .load_manifest(plugin_path)
            .map_err(crate::Error::PluginLoadError)?;

        let metadata = PluginMetadata {
            id: plugin_id.0.clone(),
            name: manifest.name.clone(),
            path: plugin_path.to_path_buf(),
            manifest,
            is_loaded: true,
            last_accessed: Some(std::time::SystemTime::now()),
            load_count: 1,
        };

        let wrapper = DenoPluginWrapper::new(plugin_id.clone(), metadata)?;
        self.loaded_plugins
            .insert(plugin_id.clone(), wrapper.clone());
        self.stats.plugins_loaded += 1;

        Ok(wrapper)
    }

    /// Process ActionItem request
    pub async fn process_action_item_request(
        &mut self,
        request: ActionItemRequest,
    ) -> Result<ActionItemResponse> {
        self.stats.requests_processed += 1;

        // Use the working request handler
        match self
            .request_handler
            .handle_action_item_request(request)
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => {
                self.stats.errors_encountered += 1;
                Err(crate::Error::RuntimeError(e))
            },
        }
    }

    /// Get runtime statistics
    pub fn get_runtime_stats(&self) -> &RuntimeStats {
        &self.stats
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.loaded_plugins
            .values()
            .map(|wrapper| wrapper.metadata().clone())
            .collect()
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, plugin_id: &PluginId) -> Result<()> {
        if self.loaded_plugins.remove(plugin_id).is_some() {
            self.plugin_manager
                .unload_plugin(&plugin_id.0)
                .map_err(crate::Error::PluginLoadError)?;
            self.stats.plugins_loaded = self.stats.plugins_loaded.saturating_sub(1);
            Ok(())
        } else {
            Err(crate::Error::PluginNotFound(plugin_id.0.clone()))
        }
    }

    /// Get request handler for external systems
    pub fn request_handler(&self) -> &RequestHandler {
        &self.request_handler
    }
}
