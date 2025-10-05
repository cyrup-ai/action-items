//! Plugin builder structure and construction

use std::sync::Arc;

use action_items_native::PluginCapabilities;

use super::built_plugin::BuiltPlugin;
use super::traits::LauncherPlugin;
use super::types::{ActionHandler, RefreshHandler, SearchHandler};
use crate::plugins::interface::PluginManifest;
use crate::service_bridge::bridge::core::ServiceBridge;

/// Fluent builder for creating launcher plugins
pub struct PluginBuilder {
    pub(super) manifest: PluginManifest,
    pub(super) search_handler: Option<SearchHandler>,
    pub(super) action_handler: Option<ActionHandler>,
    pub(super) refresh_handler: Option<RefreshHandler>,
    pub(super) service_bridge: Option<Arc<ServiceBridge>>,
}

impl PluginBuilder {
    /// Create a new plugin builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            manifest: PluginManifest {
                // Basic metadata
                id: id.into(),
                name: name.into(),
                version: "0.1.0".to_string(),
                description: String::new(),
                author: String::new(),
                license: String::new(),
                homepage: None,
                repository: None,

                // UI and presentation
                icon: None,
                categories: Vec::new(),
                keywords: Vec::new(),

                // Capabilities and permissions
                capabilities: PluginCapabilities::default(),
                permissions: Default::default(),

                // Configuration
                configuration: Vec::new(),
                preferences: Vec::new(),

                // Commands and actions
                commands: Vec::new(),
                actions: Vec::new(),

                // Dependencies
                dependencies: Default::default(),
                environment: Default::default(),

                // Update and lifecycle
                min_launcher_version: "0.1.0".to_string(),
                max_launcher_version: None,
                update_url: None,
                changelog_url: None,
            },
            search_handler: None,
            action_handler: None,
            refresh_handler: None,
            service_bridge: None,
        }
    }

    /// Set service bridge Arc reference - do NOT clone ServiceBridge
    pub fn with_service_bridge(mut self, service_bridge: Arc<ServiceBridge>) -> Self {
        self.service_bridge = Some(service_bridge);
        self
    }

    /// Build the plugin
    pub fn build(self) -> Box<dyn LauncherPlugin> {
        let service_bridge = self
            .service_bridge
            .unwrap_or_else(|| Arc::new(ServiceBridge::new()));

        Box::new(BuiltPlugin {
            manifest: self.manifest,
            search_handler: self.search_handler,
            action_handler: self.action_handler,
            refresh_handler: self.refresh_handler,
            service_bridge,
        })
    }

    /// Build the plugin and return an FFI-safe pointer (for use in extern functions)
    #[allow(dead_code)]
    pub fn build_ffi(
        self,
    ) -> Result<crate::plugins::interface::ffi::LauncherPluginFFI, crate::Error> {
        let service_bridge = self
            .service_bridge
            .clone()
            .unwrap_or_else(|| Arc::new(ServiceBridge::new()));

        // Create a new BuiltPlugin instance for FFI conversion
        let ffi_plugin_instance = BuiltPlugin {
            manifest: self.manifest.clone(),
            search_handler: self.search_handler.clone(),
            action_handler: self.action_handler.clone(),
            refresh_handler: self.refresh_handler.clone(),
            service_bridge,
        };

        // Create FFI-safe wrapper using the helper function
        let ffi_plugin = crate::plugins::interface::ffi::ffi_helpers::plugin_to_ffi(Box::new(
            ffi_plugin_instance,
        )
            as Box<dyn crate::plugins::interface::ffi::LauncherPlugin>);

        Ok(ffi_plugin)
    }
}
