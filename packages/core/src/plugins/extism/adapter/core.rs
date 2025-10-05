use std::sync::Arc;

use extism::{Plugin, PluginBuilder};

use crate::plugins::extism::manifest::create_manifest_from_data;
use crate::plugins::interface::{PluginContext, PluginManifest};

/// Adapter that wraps an Extism plugin to implement the NativePlugin trait
pub struct ExtismPluginAdapter {
    pub(super) plugin: Arc<parking_lot::Mutex<Plugin>>,
    pub(super) manifest: PluginManifest,
    pub(super) context: Arc<parking_lot::Mutex<Option<PluginContext>>>,
}

impl ExtismPluginAdapter {
    /// Create a new adapter from plugin bytes and manifest
    pub fn new(
        manifest: PluginManifest,
        plugin_data: Vec<u8>,
        host_functions: Vec<extism::Function>,
    ) -> crate::Result<Self> {
        let extism_manifest_data = create_manifest_from_data(plugin_data);
        let plugin = PluginBuilder::new(extism_manifest_data)
            .with_wasi(true)
            .with_functions(host_functions)
            .build()
            .map_err(|e| crate::Error::Extism(e.to_string()))?;

        Ok(Self {
            plugin: Arc::new(parking_lot::Mutex::new(plugin)),
            manifest,
            context: Arc::new(parking_lot::Mutex::new(None)),
        })
    }
}
