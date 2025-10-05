//! Extism plugin implementation - now decomposed into focused modules
//! 
//! This module has been refactored to improve maintainability and follows
//! the single responsibility principle with clear module boundaries.

pub mod adapter;
pub mod bridge_integration;
pub mod host_functions;
pub mod manifest;
pub mod runtime;

pub use adapter::ExtismPluginAdapter;
pub use bridge_integration::{create_host_user_data, create_plugin_context_with_bridge};
pub use host_functions::{create_host_functions, ExtismHostUserData};
pub use manifest::{create_manifest_from_data, load_manifest_from_file, validate_plugin_exports};
pub use runtime::ExtismPluginRuntime;

use std::path::Path;

use crate::plugins::interface::PluginManifest;

/// Factory function to create an Extism plugin adapter from a service bridge
pub fn create_extism_plugin_from_bridge(
    manifest: PluginManifest,
    plugin_data: Vec<u8>,
    service_bridge: &crate::plugins::bridge::ServiceBridge,
    storage_base_path: &Path,
) -> crate::Result<Box<dyn action_items_native::native::NativePlugin>> {
    let context = create_plugin_context_with_bridge(&manifest, service_bridge, storage_base_path)?;
    let host_user_data = create_host_user_data(&manifest, service_bridge, &context);
    let functions = create_host_functions(host_user_data);

    let adapter = ExtismPluginAdapter::new(manifest, plugin_data, functions)?;
    Ok(Box::new(adapter))
}