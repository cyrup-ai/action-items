//! Plugin Capability Indexing System
//!
//! Blazing-fast capability lookups with O(1) hash-based operations
//! and O(log n) prefix searching using optimized data structures.

use std::collections::BTreeMap;

use bevy::prelude::*;
use rustc_hash::FxHashMap;

use crate::components::Capability;

/// Plugin capability index for blazing-fast capability lookups
#[derive(Resource)]
pub struct PluginCapabilityIndex {
    /// Capability name -> Plugin IDs that provide it
    pub capability_to_plugins: FxHashMap<String, Vec<String>>,
    /// Plugin ID -> Capabilities it provides
    pub plugin_to_capabilities: FxHashMap<String, Vec<String>>,
    /// Sorted capability names for efficient searching
    pub sorted_capabilities: BTreeMap<String, Vec<String>>,
}

impl Default for PluginCapabilityIndex {
    #[inline]
    fn default() -> Self {
        Self {
            capability_to_plugins: FxHashMap::default(),
            plugin_to_capabilities: FxHashMap::default(),
            sorted_capabilities: BTreeMap::new(),
        }
    }
}

impl PluginCapabilityIndex {
    /// Create a new PluginCapabilityIndex
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify capability access for a plugin
    pub fn verify_capability(&mut self, plugin_id: &str, capability: &str) -> Result<bool, String> {
        if let Some(capabilities) = self.plugin_to_capabilities.get(plugin_id) {
            Ok(capabilities.contains(&capability.to_string()))
        } else {
            Err(format!(
                "Plugin {} not found in capability index",
                plugin_id
            ))
        }
    }

    /// Add plugin capabilities with O(log n) complexity
    #[inline]
    pub fn add_plugin_capabilities(&mut self, plugin_id: String, capabilities: Vec<Capability>) {
        let capability_names: Vec<String> = capabilities.iter().map(|c| c.name.clone()).collect();

        // Update plugin -> capabilities mapping
        self.plugin_to_capabilities
            .insert(plugin_id.clone(), capability_names.clone());

        // Update capability -> plugins mapping
        for capability_name in &capability_names {
            self.capability_to_plugins
                .entry(capability_name.clone())
                .or_default()
                .push(plugin_id.clone());

            // Update sorted index
            self.sorted_capabilities
                .entry(capability_name.clone())
                .or_default()
                .push(plugin_id.clone());
        }
    }

    /// Remove plugin from index
    #[inline]
    pub fn remove_plugin(&mut self, plugin_id: &str) {
        if let Some(capabilities) = self.plugin_to_capabilities.remove(plugin_id) {
            for capability_name in capabilities {
                if let Some(plugins) = self.capability_to_plugins.get_mut(&capability_name) {
                    plugins.retain(|id| id != plugin_id);
                    if plugins.is_empty() {
                        self.capability_to_plugins.remove(&capability_name);
                    }
                }

                if let Some(plugins) = self.sorted_capabilities.get_mut(&capability_name) {
                    plugins.retain(|id| id != plugin_id);
                    if plugins.is_empty() {
                        self.sorted_capabilities.remove(&capability_name);
                    }
                }
            }
        }
    }

    /// Find plugins by capability with O(1) lookup
    #[inline]
    pub fn find_plugins_by_capability(&self, capability: &str) -> Option<&Vec<String>> {
        self.capability_to_plugins.get(capability)
    }

    /// Find capabilities by plugin with O(1) lookup
    #[inline]
    pub fn find_capabilities_by_plugin(&self, plugin_id: &str) -> Option<&Vec<String>> {
        self.plugin_to_capabilities.get(plugin_id)
    }

    /// Search capabilities by prefix with O(log n) complexity
    #[inline]
    pub fn search_capabilities(&self, prefix: &str) -> Vec<&String> {
        self.sorted_capabilities
            .range(prefix.to_string()..)
            .take_while(|(key, _)| key.starts_with(prefix))
            .map(|(key, _)| key)
            .collect()
    }
}
