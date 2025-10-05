use std::path::Path;

use extism::{Manifest, Plugin, PluginBuilder, Wasm};
use log::debug;

use crate::plugins::interface::PluginManifest;
use crate::plugins::interface::extism::REQUIRED_EXPORTS;

/// Validate and load plugin manifest from WASM file
pub fn load_manifest_from_file(path: &Path) -> Result<(Plugin, PluginManifest), crate::Error> {
    // Create Extism manifest
    let url = Wasm::file(path);
    let manifest_data = Manifest::new([url]);

    // Create plugin with host functions
    let mut plugin = PluginBuilder::new(manifest_data)
        .with_wasi(true)
        .build()
        .map_err(|e| crate::Error::Extism(e.to_string()))?;

    // Get plugin manifest
    let manifest_json = plugin
        .call::<(), String>("plugin_manifest", ())
        .map_err(|e| crate::Error::Extism(e.to_string()))?;
    let plugin_manifest: PluginManifest = serde_json::from_str(&manifest_json)?;

    // Validate required exports
    validate_plugin_exports(&plugin)?;

    debug!(
        "Successfully loaded manifest for plugin: {}",
        plugin_manifest.id
    );
    Ok((plugin, plugin_manifest))
}

/// Validate that plugin has all required exports
pub fn validate_plugin_exports(plugin: &Plugin) -> Result<(), crate::Error> {
    for export in REQUIRED_EXPORTS {
        if !plugin.function_exists(export) {
            return Err(crate::Error::PluginError(format!(
                "Plugin missing required export: {export}"
            )));
        }
    }
    Ok(())
}

/// Create Extism manifest from plugin data
pub fn create_manifest_from_data(plugin_data: Vec<u8>) -> Manifest {
    let wasm = Wasm::data(plugin_data);
    Manifest::new([wasm])
}
