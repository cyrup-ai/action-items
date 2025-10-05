use std::collections::HashMap;
use std::path::Path;

use action_items_native::{
    CacheService, ClipboardAccess, HttpClient, NotificationService, PluginContext, PluginManifest,
    StorageService,
};

use crate::plugins::extism::host_functions::ExtismHostUserData;

/// Create plugin context with service bridge integration
pub fn create_plugin_context_with_bridge(
    manifest: &PluginManifest,
    _service_bridge: &crate::service_bridge::bridge::core::ServiceBridge,
    storage_base_path: &Path,
) -> Result<PluginContext, String> {
    let plugin_specific_storage_dir = storage_base_path.join(&manifest.id);
    std::fs::create_dir_all(&plugin_specific_storage_dir).map_err(|e| {
        format!(
            "Failed to create plugin storage directory for {}: {}",
            manifest.id, e
        )
    })?;

    let (storage_read_sender, _storage_read_receiver) = crossbeam_channel::unbounded();
    let (storage_write_sender, _storage_write_receiver) = crossbeam_channel::unbounded();

    // Create enhanced context with bridge integration
    let context = PluginContext {
        plugin_id: manifest.id.clone(),
        config: HashMap::new(),
        preferences: HashMap::new(),
        environment: manifest.environment.clone(),
        clipboard: ClipboardAccess::new(),
        notifications: NotificationService::new("action-items".to_string()),
        storage: StorageService::new(plugin_specific_storage_dir, manifest.id.clone())
            .map_err(|e| format!("Failed to create storage service: {:?}", e))?,
        http: HttpClient::new(std::time::Duration::from_secs(30), 3),
        cache: CacheService::new(1000),
        storage_read_sender,
        storage_write_sender,
    };

    log::debug!(
        "Created plugin context with service bridge integration for plugin '{}'",
        manifest.id
    );

    Ok(context)
}

/// Create host user data for Extism plugins
pub fn create_host_user_data(
    manifest: &PluginManifest,
    _context: &PluginContext,
) -> ExtismHostUserData {
    // Create channels for plugin communication
    let (clipboard_read_sender, _clipboard_read_receiver) = crossbeam_channel::unbounded();
    let (clipboard_write_sender, _clipboard_write_receiver) = crossbeam_channel::unbounded();
    let (notification_sender, _notification_receiver) = crossbeam_channel::unbounded();
    let (http_sender, _http_receiver) = crossbeam_channel::unbounded();
    let (storage_read_sender, _storage_read_receiver) = crossbeam_channel::unbounded();
    let (storage_write_sender, _storage_write_receiver) = crossbeam_channel::unbounded();

    ExtismHostUserData {
        plugin_id: manifest.id.clone(),
        storage_read_sender,
        storage_write_sender,
        clipboard_read_sender,
        clipboard_write_sender,
        notification_sender,
        http_sender,
        cache_service: CacheService::new(1000),
    }
}
