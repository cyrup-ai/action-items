//! Main wrapper implementation and constructor

use std::sync::Arc;

use parking_lot::RwLock;

// Note: PluginManifest and info logging will be used when wrapper implementation is completed
use super::types::{NativePluginWrapper, PluginMetadata};
use crate::error::Result;
use crate::plugins::interface::NativePlugin;

impl NativePluginWrapper {
    /// Create a new wrapper around a NativePlugin
    pub fn new(plugin: Box<dyn NativePlugin>) -> Result<Self> {
        let manifest = plugin.manifest().clone();
        let metadata = PluginMetadata {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            capabilities: {
                let caps = &manifest.capabilities;
                let mut capability_list = Vec::new();
                if caps.search {
                    capability_list.push("search".to_string());
                }
                if caps.background_refresh {
                    capability_list.push("background_refresh".to_string());
                }
                if caps.notifications {
                    capability_list.push("notifications".to_string());
                }
                if caps.shortcuts {
                    capability_list.push("shortcuts".to_string());
                }
                if caps.deep_links {
                    capability_list.push("deep_links".to_string());
                }
                if caps.clipboard_access {
                    capability_list.push("clipboard_access".to_string());
                }
                if caps.file_system_access {
                    capability_list.push("file_system_access".to_string());
                }
                if caps.network_access {
                    capability_list.push("network_access".to_string());
                }
                if caps.system_commands {
                    capability_list.push("system_commands".to_string());
                }
                if caps.ui_extensions {
                    capability_list.push("ui_extensions".to_string());
                }
                if caps.context_menu {
                    capability_list.push("context_menu".to_string());
                }
                if caps.quick_actions {
                    capability_list.push("quick_actions".to_string());
                }
                capability_list
            },
            description: manifest.description.clone(),
            version: manifest.version.clone(),
            manifest,
        };

        Ok(Self {
            plugin: Arc::new(RwLock::new(plugin)),
            metadata,
        })
    }

    /// Get the plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get a reference to the wrapped plugin
    pub fn plugin(&self) -> Arc<RwLock<Box<dyn NativePlugin>>> {
        self.plugin.clone()
    }
}
