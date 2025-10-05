use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::plugins::extism::wrapper::ExtismPluginComponent;
use crate::plugins::native::wrapper::PluginComponent;
use crate::raycast::wrapper::RaycastPluginComponent;

/// ECS-based plugin name collector that replaces PluginRegistry.plugin_names()
///
/// Usage: `fn my_system(plugin_names: PluginNames) { let names = plugin_names.collect(); }`
#[derive(SystemParam)]
pub struct PluginNames<'w, 's> {
    native_plugins: Query<'w, 's, &'static PluginComponent>,
    extism_plugins: Query<'w, 's, &'static ExtismPluginComponent>,
    raycast_plugins: Query<'w, 's, &'static RaycastPluginComponent>,
}

impl<'w, 's> PluginNames<'w, 's> {
    /// Collect all plugin names into a vector
    pub fn collect(&self) -> Vec<String> {
        let mut names = Vec::new();

        // Collect native plugin names
        for plugin in self.native_plugins.iter() {
            names.push(plugin.name.clone());
        }

        // Collect extism plugin names
        for plugin in self.extism_plugins.iter() {
            names.push(plugin.name.clone());
        }

        // Collect raycast plugin names
        for plugin in self.raycast_plugins.iter() {
            names.push(plugin.name.clone());
        }

        names.sort();
        names
    }

    /// Collect native plugin names only
    pub fn native_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .native_plugins
            .iter()
            .map(|plugin| plugin.name.clone())
            .collect();
        names.sort();
        names
    }

    /// Collect extism plugin names only
    pub fn extism_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .extism_plugins
            .iter()
            .map(|plugin| plugin.name.clone())
            .collect();
        names.sort();
        names
    }

    /// Collect raycast plugin names only
    pub fn raycast_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .raycast_plugins
            .iter()
            .map(|plugin| plugin.name.clone())
            .collect();
        names.sort();
        names
    }

    /// Check if a plugin with the given name exists
    pub fn contains(&self, name: &str) -> bool {
        self.native_plugins.iter().any(|plugin| plugin.name == name)
            || self.extism_plugins.iter().any(|plugin| plugin.name == name)
            || self
                .raycast_plugins
                .iter()
                .any(|plugin| plugin.name == name)
    }

    /// Find plugins matching a name pattern
    pub fn find_matching(&self, pattern: &str) -> Vec<String> {
        let mut matching = Vec::new();

        // Check native plugins
        for plugin in self.native_plugins.iter() {
            if plugin.name.contains(pattern) {
                matching.push(plugin.name.clone());
            }
        }

        // Check extism plugins
        for plugin in self.extism_plugins.iter() {
            if plugin.name.contains(pattern) {
                matching.push(plugin.name.clone());
            }
        }

        // Check raycast plugins
        for plugin in self.raycast_plugins.iter() {
            if plugin.name.contains(pattern) {
                matching.push(plugin.name.clone());
            }
        }

        matching.sort();
        matching.dedup();
        matching
    }

    /// Get plugin names grouped by type
    pub fn grouped_by_type(&self) -> PluginNameGroups {
        PluginNameGroups {
            native: self.native_names(),
            extism: self.extism_names(),
            raycast: self.raycast_names(),
        }
    }
}

/// Plugin names grouped by type
#[derive(Debug, Clone)]
pub struct PluginNameGroups {
    pub native: Vec<String>,
    pub extism: Vec<String>,
    pub raycast: Vec<String>,
}

impl PluginNameGroups {
    /// Get all names as a flat vector
    pub fn all_names(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(self.native.clone());
        all.extend(self.extism.clone());
        all.extend(self.raycast.clone());
        all.sort();
        all
    }

    /// Get total count across all types
    pub fn total_count(&self) -> usize {
        self.native.len() + self.extism.len() + self.raycast.len()
    }
}
