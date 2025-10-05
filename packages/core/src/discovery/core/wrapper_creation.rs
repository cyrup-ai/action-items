use std::path::Path;

use action_items_common::plugin_interface::PluginManifest;
use log::info;

use super::build_management::find_or_build_plugin_library;
use super::types::DiscoveredPlugin;
use crate::discovery::adapter::FFIToNativeAdapter;
use crate::discovery::loader::{DynamicPlugin, is_native_plugin_file};
use crate::plugins::extism::ExtismPluginAdapter;
use crate::plugins::extism::wrapper::ExtismPluginWrapper;
use crate::plugins::native::wrapper::NativePluginWrapper;
use crate::{Error, Result};

/// Creates a minimal plugin manifest for discovered plugins
pub fn create_minimal_manifest(plugin_name: &str, description: &str) -> PluginManifest {
    use std::collections::HashMap;

    use action_items_common::plugin_interface::{
        PluginCapabilities, PluginCategory, PluginPermissions,
    };

    PluginManifest {
        id: plugin_name.to_string(),
        name: plugin_name.to_string(),
        version: "1.0.0".to_string(),
        description: description.to_string(),
        author: "Unknown".to_string(),
        license: "Unknown".to_string(),
        homepage: None,
        repository: None,
        icon: None,
        categories: vec![PluginCategory::Custom("plugin".to_string())],
        keywords: vec!["plugin".to_string()],
        capabilities: PluginCapabilities::default(),
        permissions: PluginPermissions::default(),
        configuration: vec![],
        preferences: vec![],
        commands: vec![],
        actions: vec![],
        dependencies: HashMap::new(),
        environment: HashMap::new(),
        min_launcher_version: "0.1.0".to_string(),
        max_launcher_version: None,
        update_url: None,
        changelog_url: None,
    }
}

/// Creates a plugin wrapper from a file (WASM or native dynamic library)
pub fn create_plugin_wrapper_from_file(path: &Path) -> Result<DiscoveredPlugin> {
    let plugin_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    if path.extension().and_then(|ext| ext.to_str()) == Some("wasm") {
        // Load WASM plugin data and create ExtismPluginAdapter
        let plugin_data = std::fs::read(path)
            .map_err(|e| Error::PluginLoadError(format!("Failed to read WASM file: {e}")))?;

        let manifest = create_minimal_manifest(
            &plugin_name,
            &format!("WASM plugin loaded from {}", path.display()),
        );

        // Create ExtismPluginAdapter with manifest, plugin data, and host functions
        let (storage_read_sender, _storage_read_receiver) = crossbeam_channel::unbounded();
        let (storage_write_sender, _storage_write_receiver) = crossbeam_channel::unbounded();
        let context = action_items_native::PluginContext {
            plugin_id: manifest.id.clone(),
            config: std::collections::HashMap::new(),
            preferences: std::collections::HashMap::new(),
            environment: manifest.environment.clone(),
            clipboard: action_items_native::context::ClipboardAccess::new(),
            notifications: action_items_native::context::NotificationService::new(
                "action-items".to_string(),
            ),
            storage: action_items_native::context::StorageService::new(
                std::path::PathBuf::from("./storage").join(&manifest.id),
                manifest.id.clone(),
            )
            .map_err(|e| {
                Error::PluginLoadError(format!("Failed to create storage service: {:?}", e))
            })?,
            http: action_items_native::context::HttpClient::new(
                std::time::Duration::from_secs(30),
                3,
            ),
            cache: action_items_native::context::CacheService::new(1000),
            storage_read_sender,
            storage_write_sender,
        };
        let host_user_data = crate::plugins::extism::create_host_user_data(&manifest, &context);
        let host_functions =
            crate::plugins::extism::host_functions::create_host_functions(host_user_data);
        let adapter = ExtismPluginAdapter::new(manifest.clone(), plugin_data, host_functions)?;
        let wrapper = ExtismPluginWrapper::new(adapter, manifest)?;
        Ok(DiscoveredPlugin::Extism(wrapper))
    } else if is_native_plugin_file(path) {
        // Load native plugin dynamically and create NativePluginWrapper
        let dynamic_plugin = DynamicPlugin::load_from_path(path)?;
        let plugin = dynamic_plugin.into_plugin();
        let adapter = FFIToNativeAdapter::new(plugin);
        let wrapper = NativePluginWrapper::new(Box::new(adapter))?;
        Ok(DiscoveredPlugin::Native(wrapper))
    } else {
        Err(Error::PluginLoadError(format!(
            "Unknown plugin file type: {}",
            path.display()
        )))
    }
}

/// Creates a plugin wrapper from a Rust plugin project directory
pub fn create_plugin_wrapper_from_rust_project(dir: &Path) -> Result<DiscoveredPlugin> {
    info!("Found Rust plugin project at {}", dir.display());

    // Build the plugin and get the library path
    let lib_path = find_or_build_plugin_library(dir)?;

    // Create wrapper from the built library
    create_plugin_wrapper_from_file(&lib_path)
}
