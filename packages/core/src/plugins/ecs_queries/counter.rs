use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::plugins::extism::wrapper::ExtismPluginComponent;
use crate::plugins::native::wrapper::PluginComponent;
use crate::raycast::wrapper::RaycastPluginComponent;

/// ECS-based plugin counter that replaces PluginRegistry.plugin_count()
///
/// This SystemParam groups all plugin queries together for easy access.
/// Usage: `fn my_system(plugin_counter: PluginCounter) { let count = plugin_counter.count(); }`
#[derive(SystemParam)]
pub struct PluginCounter<'w, 's> {
    native_plugins: Query<'w, 's, &'static PluginComponent>,
    extism_plugins: Query<'w, 's, &'static ExtismPluginComponent>,
    raycast_plugins: Query<'w, 's, &'static RaycastPluginComponent>,
}

impl<'w, 's> PluginCounter<'w, 's> {
    /// Get total count of all loaded plugins
    pub fn count(&self) -> usize {
        self.native_plugins.iter().count()
            + self.extism_plugins.iter().count()
            + self.raycast_plugins.iter().count()
    }

    /// Get count by plugin type
    pub fn native_count(&self) -> usize {
        self.native_plugins.iter().count()
    }

    pub fn extism_count(&self) -> usize {
        self.extism_plugins.iter().count()
    }

    pub fn raycast_count(&self) -> usize {
        self.raycast_plugins.iter().count()
    }

    /// Check if any plugins are loaded
    pub fn has_plugins(&self) -> bool {
        self.count() > 0
    }

    /// Check if specific plugin type is loaded
    pub fn has_native_plugins(&self) -> bool {
        self.native_count() > 0
    }

    pub fn has_extism_plugins(&self) -> bool {
        self.extism_count() > 0
    }

    pub fn has_raycast_plugins(&self) -> bool {
        self.raycast_count() > 0
    }

    /// Get plugin type distribution as percentages
    pub fn type_distribution(&self) -> PluginTypeDistribution {
        let total = self.count();
        if total == 0 {
            return PluginTypeDistribution::default();
        }

        let native_percent = (self.native_count() as f32 / total as f32) * 100.0;
        let extism_percent = (self.extism_count() as f32 / total as f32) * 100.0;
        let raycast_percent = (self.raycast_count() as f32 / total as f32) * 100.0;

        PluginTypeDistribution {
            native_percent,
            extism_percent,
            raycast_percent,
        }
    }
}

/// Plugin type distribution statistics
#[derive(Debug, Clone, Default)]
pub struct PluginTypeDistribution {
    pub native_percent: f32,
    pub extism_percent: f32,
    pub raycast_percent: f32,
}

impl PluginTypeDistribution {
    /// Get the dominant plugin type
    pub fn dominant_type(&self) -> PluginType {
        if self.native_percent >= self.extism_percent && self.native_percent >= self.raycast_percent
        {
            PluginType::Native
        } else if self.extism_percent >= self.raycast_percent {
            PluginType::Extism
        } else {
            PluginType::Raycast
        }
    }
}

/// Plugin type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    Native,
    Extism,
    Raycast,
}
