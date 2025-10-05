//! Core types and metadata structures for native plugin wrapper

use std::path::Path;
use std::sync::Arc;

use action_items_common::plugin_interface::PluginManifest;
use bevy::prelude::*;
use parking_lot::RwLock;

use crate::discovery::core::types::MetadataProvider;
use crate::plugins::interface::NativePlugin;

/// Bevy Plugin wrapper around NativePlugin
///
/// This wrapper allows existing NativePlugin implementations to be registered
/// as proper Bevy plugins, enabling them to participate in the Bevy ECS lifecycle
/// without changing the existing plugin trait interfaces.
#[derive(Clone)]
pub struct NativePluginWrapper {
    /// The wrapped native plugin instance
    pub(super) plugin: Arc<RwLock<Box<dyn NativePlugin>>>,
    /// Plugin metadata for registration and discovery
    pub(super) metadata: PluginMetadata,
}

/// Metadata required for plugin registration
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub manifest: PluginManifest,
    pub capabilities: Vec<String>,
    pub description: String,
    pub version: String,
}

impl MetadataProvider for PluginMetadata {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_version(&self) -> &str {
        &self.version
    }

    fn get_path(&self) -> Option<&Path> {
        None // Native plugins don't have file paths in this metadata
    }
}

/// Component that holds plugin instance and metadata
#[derive(Component)]
pub struct PluginComponent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub config: PluginMetadata,
    pub plugin: Arc<RwLock<Box<dyn NativePlugin>>>,
}
