use std::path::Path;
use std::sync::Arc;

use action_items_common::plugin_interface::PluginManifest;
use bevy::prelude::*;
use log::{debug, info};
use parking_lot::RwLock;

use crate::discovery::core::types::MetadataProvider;
use crate::error::Result;
use crate::plugins::extism::ExtismPluginAdapter;
use crate::search::{SearchIndex, SearchItem, SearchItemType};

/// Bevy Plugin wrapper around Extism WASM Plugin
///
/// This wrapper allows existing Extism WASM plugins to be registered
/// as proper Bevy plugins, enabling them to participate in the Bevy ECS lifecycle
/// without changing the existing plugin loading mechanisms.
#[derive(Clone)]
pub struct ExtismPluginWrapper {
    /// The wrapped Extism plugin adapter that implements NativePlugin
    adapter: Arc<RwLock<ExtismPluginAdapter>>,
    /// Plugin metadata for registration and discovery
    metadata: PluginMetadata,
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
        None // Extism plugins don't have file paths in this metadata
    }
}

impl ExtismPluginWrapper {
    /// Create a new wrapper around an Extism Plugin
    pub fn new(adapter: ExtismPluginAdapter, manifest: PluginManifest) -> Result<Self> {
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
            adapter: Arc::new(RwLock::new(adapter)),
            metadata,
        })
    }

    /// Get the plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get a reference to the wrapped plugin adapter
    pub fn adapter(&self) -> Arc<RwLock<ExtismPluginAdapter>> {
        self.adapter.clone()
    }
}

impl Plugin for ExtismPluginWrapper {
    fn build(&self, app: &mut App) {
        let metadata = self.metadata.clone();
        let adapter = self.adapter.clone();
        
        // Use startup system to spawn entity instead of direct world spawning
        app.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(ExtismPluginComponent {
                id: metadata.id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                plugin: adapter.clone(),
            });
        });

        // Move SearchIndex operations to startup system to avoid CommandQueue issues
        let metadata_for_search = self.metadata.clone();
        
        app.add_systems(Startup, move |mut search_index: ResMut<SearchIndex>| {
            // Create search items from plugin manifest
            let manifest = &metadata_for_search.manifest;

            // Add main plugin entry
            let plugin_item = SearchItem::new(
                format!("extism:{}", metadata_for_search.id),
                metadata_for_search.name.clone(),
                manifest.description.clone(),
                SearchItemType::Plugin,
            )
            .with_keywords(manifest.keywords.clone());

            search_index.add_item(plugin_item);

            // Add action entries
            for action in &manifest.actions {
                let action_item = SearchItem::new(
                    format!("extism:{}:{}", metadata_for_search.id, action.id),
                    action.title.clone(),
                    action.description.clone().unwrap_or_default(),
                    SearchItemType::ActionItem,
                );

                search_index.add_item(action_item);
            }

            info!(
                "Added Extism plugin '{}' with {} actions to search index",
                metadata_for_search.name,
                manifest.actions.len()
            );
        });
    }
}

/// Component that holds Extism plugin instance and metadata
#[derive(Component)]
pub struct ExtismPluginComponent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub plugin: Arc<RwLock<ExtismPluginAdapter>>,
}

/// System to handle Extism plugin execution requests
pub fn execute_extism_plugin_system(
    plugins: Query<&ExtismPluginComponent>,
    // Add event readers for plugin execution when available
) {
    // This system will handle execution requests for Extism plugins
    // Implementation will depend on the event system design
    for plugin_component in &plugins {
        // Plugin execution logic will be added based on event system
        debug!("Extism plugin available: {}", plugin_component.name);
    }
}
