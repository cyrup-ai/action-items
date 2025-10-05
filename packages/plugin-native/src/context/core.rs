use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crossbeam_channel::Sender as CrossbeamSender;
use serde_json::Value;

use super::requests::{StorageReadRequest, StorageWriteRequest};
use super::services::{
    CacheService, ClipboardAccess, HttpClient, NotificationService, StorageService,
};

/// Result type for plugin command execution with zero-allocation patterns
/// Wraps a serde_json::Value to allow for flexible return types
pub type CommandResult = Value;

/// Plugin context provided to commands using modern event-driven architecture
/// This is a lightweight context that provides access to plugin configuration
/// Actual service operations are performed through Bevy systems and events
#[derive(Clone, serde::Serialize)]
pub struct PluginContext {
    pub plugin_id: String,
    pub config: HashMap<String, serde_json::Value>,
    pub preferences: HashMap<String, serde_json::Value>,
    pub environment: HashMap<String, String>,
    #[serde(skip)]
    pub clipboard: ClipboardAccess,
    #[serde(skip)]
    pub notifications: NotificationService,
    #[serde(skip)]
    pub storage: StorageService, // Direct access for native, events for Extism host fns
    #[serde(skip)]
    pub http: HttpClient,
    #[serde(skip)]
    pub cache: CacheService,
    // Modern event-driven senders for zero-allocation communication
    #[serde(skip)]
    pub storage_read_sender: CrossbeamSender<StorageReadRequest>,
    #[serde(skip)]
    pub storage_write_sender: CrossbeamSender<StorageWriteRequest>,
}

impl PluginContext {
    /// Create plugin context with functional service bridge integration
    /// This is the ONLY way to create a PluginContext with functional channels
    pub fn with_service_bridge(
        plugin_id: String,
        storage_read_sender: CrossbeamSender<StorageReadRequest>,
        storage_write_sender: CrossbeamSender<StorageWriteRequest>,
        plugin_data_dir: PathBuf,
    ) -> Self {
        let default_storage = plugin_data_dir.join(&plugin_id);

        Self {
            plugin_id: plugin_id.clone(),
            config: HashMap::new(),
            preferences: HashMap::new(),
            environment: HashMap::new(),
            clipboard: ClipboardAccess::new(),
            notifications: NotificationService::new("Action Items".to_string()),
            storage: StorageService::new(plugin_data_dir.clone(), plugin_id.clone()).unwrap_or(
                StorageService {
                    base_path: default_storage,
                },
            ),
            http: HttpClient::new(Duration::from_secs(30), 3),
            cache: CacheService::new(1000),
            storage_read_sender,
            storage_write_sender,
        }
    }
}
