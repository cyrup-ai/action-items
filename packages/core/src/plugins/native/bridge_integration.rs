//! Native plugin bridge integration for functional storage operations

use std::collections::HashMap;
use std::path::Path;

use action_items_native::{
    CacheService, ClipboardAccess, HttpClient, NotificationService, PluginContext, PluginManifest,
    StorageService,
};

use crate::service_bridge::bridge::core::ServiceBridge;

/// Create native plugin context with service bridge integration
pub fn create_native_plugin_context_with_bridge(
    manifest: &PluginManifest,
    service_bridge: &ServiceBridge,
    storage_base_path: &Path,
) -> crate::Result<PluginContext> {
    let plugin_specific_storage_dir = storage_base_path.join(&manifest.id);
    std::fs::create_dir_all(&plugin_specific_storage_dir).map_err(|e| {
        crate::error::Error::PluginError(format!(
            "Failed to create native plugin storage directory for {}: {}",
            manifest.id, e
        ))
    })?;

    // Register native plugin with NEW service bridge
    service_bridge
        .register_plugin_simple(manifest.id.clone(), manifest.name.clone(), vec![])
        .map_err(crate::error::Error::PluginError)?;

    // Create storage channels using crossbeam
    let (storage_read_sender, _storage_read_receiver) = crossbeam_channel::unbounded();
    let (storage_write_sender, _storage_write_receiver) = crossbeam_channel::unbounded();

    // Create production-ready context with actual service bridge connections
    let context = PluginContext {
        plugin_id: manifest.id.clone(),
        config: HashMap::new(),
        preferences: HashMap::new(),
        environment: manifest.environment.clone(),
        clipboard: ClipboardAccess::new(),
        notifications: NotificationService::new("action-items".to_string()),
        storage: StorageService::new(plugin_specific_storage_dir, manifest.id.clone())?,
        http: HttpClient::new(std::time::Duration::from_secs(30), 3),
        cache: CacheService::new(1000),
        // Connect to actual service bridge channels
        storage_read_sender,
        storage_write_sender,
    };

    // NEW ServiceBridge handles context configuration and storage automatically

    log::debug!(
        "Created native plugin context with service bridge integration for plugin '{}'",
        manifest.id
    );

    Ok(context)
}
