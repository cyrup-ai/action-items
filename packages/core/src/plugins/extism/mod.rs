//! Extism plugin support for WASM-based plugins
//!
//! This module provides a complete implementation for running Extism WASM plugins
//! with full host function support and service bridge integration.

pub mod adapter;
pub mod bridge_integration;
pub mod host_functions;
pub mod manifest;
pub mod runtime;
pub mod wrapper;

use std::path::Path;

pub use adapter::ExtismPluginAdapter;
// Re-export types for compatibility
pub use adapter::ExtismPluginAdapter as ExtismPlugin;
pub use bridge_integration::{create_host_user_data, create_plugin_context_with_bridge};
pub use host_functions::{ExtismHostUserData, create_host_functions};
pub use manifest::{create_manifest_from_data, load_manifest_from_file, validate_plugin_exports};
pub use runtime::{ExtismPluginRuntime, ExtismPluginRuntime as ExtismPluginLoader};
pub use wrapper::{ExtismPluginComponent, ExtismPluginWrapper};

use crate::plugins::interface::PluginManifest;

/// Factory function to create an Extism plugin adapter from a service bridge
pub fn create_extism_plugin_from_bridge(
    manifest: PluginManifest,
    plugin_data: Vec<u8>,
    service_bridge: &crate::service_bridge::bridge::core::ServiceBridge,
    storage_base_path: &Path,
) -> crate::Result<Box<dyn action_items_native::native::NativePlugin>> {
    let context = create_plugin_context_with_bridge(&manifest, service_bridge, storage_base_path)
        .map_err(crate::Error::PluginError)?;
    let host_user_data = create_host_user_data(&manifest, &context);
    let functions = create_host_functions(host_user_data);

    let adapter = ExtismPluginAdapter::new(manifest, plugin_data, functions)?;
    Ok(Box::new(adapter))
}
