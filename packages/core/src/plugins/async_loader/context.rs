use std::collections::HashMap;

use action_items_native::context::{
    CacheService, ClipboardAccess, HttpClient, NotificationService, StorageService,
};
use action_items_native::{PluginContext, PluginManifest};

use crate::raycast::wrapper::PluginMetadata as RaycastMetadata;

/// Creates a plugin context for initialization with zero allocations
#[inline]
pub fn create_plugin_context(
    manifest: &PluginManifest,
    app_directories: &crate::config::AppDirectories,
) -> Result<PluginContext, String> {
    use crossbeam_channel::unbounded;

    let plugin_storage_dir = app_directories.plugin_state().join(&manifest.id);
    std::fs::create_dir_all(&plugin_storage_dir)
        .map_err(|e| format!("Failed to create plugin storage directory: {}", e))?;

    let storage = StorageService::new(plugin_storage_dir, manifest.id.clone())
        .map_err(|e| format!("Failed to create storage service: {}", e))?;

    let (storage_read_sender, _storage_read_receiver) = unbounded();
    let (storage_write_sender, _storage_write_receiver) = unbounded();
    // Storage receivers are handled by the storage service internally

    Ok(PluginContext {
        plugin_id: manifest.id.clone(),
        config: HashMap::new(),
        preferences: HashMap::new(),
        environment: manifest.environment.clone(),
        clipboard: ClipboardAccess::new(),
        notifications: NotificationService::new(manifest.name.clone()),
        storage,
        http: HttpClient::new(std::time::Duration::from_secs(30), 3),
        cache: CacheService::new(1000),
        storage_read_sender,
        storage_write_sender,
    })
}

/// Creates a plugin context for Raycast plugins with zero allocations
#[inline]
pub fn create_raycast_plugin_context(
    metadata: &RaycastMetadata,
    app_directories: &crate::config::AppDirectories,
) -> Result<PluginContext, String> {
    use crossbeam_channel::unbounded;

    let plugin_storage_dir = app_directories.plugin_state().join(&metadata.id);
    std::fs::create_dir_all(&plugin_storage_dir)
        .map_err(|e| format!("Failed to create plugin storage directory: {}", e))?;

    let storage = StorageService::new(plugin_storage_dir, metadata.id.clone())
        .map_err(|e| format!("Failed to create storage service: {}", e))?;

    let (storage_read_sender, _storage_read_receiver) = unbounded();
    let (storage_write_sender, _storage_write_receiver) = unbounded();
    // Storage receivers are handled by the storage service internally

    Ok(PluginContext {
        plugin_id: metadata.id.clone(),
        config: HashMap::new(),
        preferences: HashMap::new(),
        environment: HashMap::new(),
        clipboard: ClipboardAccess::new(),
        notifications: NotificationService::new(metadata.name.clone()),
        storage,
        http: HttpClient::new(std::time::Duration::from_secs(30), 3),
        cache: CacheService::new(1000),
        storage_read_sender,
        storage_write_sender,
    })
}

/// Create plugin context for Deno plugins
pub(crate) fn create_deno_plugin_context(
    metadata: &crate::plugins::core::metadata::PluginMetadata,
    app_directories: &crate::config::AppDirectories,
) -> Result<PluginContext, String> {
    use crossbeam_channel::unbounded;

    let plugin_storage_dir = app_directories.plugin_state().join(&metadata.id);
    std::fs::create_dir_all(&plugin_storage_dir)
        .map_err(|e| format!("Failed to create plugin storage directory: {}", e))?;

    let storage = StorageService::new(plugin_storage_dir, metadata.id.clone())
        .map_err(|e| format!("Failed to create storage service: {}", e))?;

    let (storage_read_sender, _storage_read_receiver) = unbounded();
    let (storage_write_sender, _storage_write_receiver) = unbounded();
    // Storage receivers are handled by the storage service internally

    Ok(PluginContext {
        plugin_id: metadata.id.clone(),
        config: HashMap::new(),
        preferences: HashMap::new(),
        environment: HashMap::new(),
        clipboard: ClipboardAccess::new(),
        notifications: NotificationService::new(metadata.name.clone()),
        storage,
        http: HttpClient::new(std::time::Duration::from_secs(30), 3),
        cache: CacheService::new(1000),
        storage_read_sender,
        storage_write_sender,
    })
}
