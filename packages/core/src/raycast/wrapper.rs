use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use log::{debug, info};

use crate::discovery::core::types::MetadataProvider;
use crate::error::Result;
use crate::raycast::loader::RaycastExtension;
use crate::runtime::deno::{DenoRuntime, RuntimeChannels, RuntimeConfig};
use crate::search::{SearchIndex, SearchItem, SearchItemType};

/// Bevy Plugin wrapper around Raycast Extension
///
/// This wrapper allows existing Raycast extensions to be registered
/// as proper Bevy plugins, enabling them to participate in the Bevy ECS lifecycle
/// without changing the existing extension loading mechanisms.
#[derive(Clone)]
pub struct RaycastPluginWrapper {
    /// The wrapped Raycast extension
    extension: RaycastExtension,
    /// Plugin metadata for registration and discovery
    metadata: PluginMetadata,
}

/// Metadata required for plugin registration
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub keywords: Vec<String>,
    pub version: String,
    pub commands: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
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
        Some(&self.path)
    }
}

impl RaycastPluginWrapper {
    /// Create a new wrapper around a Raycast Extension
    pub fn new(extension: RaycastExtension) -> Result<Self> {
        let metadata = PluginMetadata {
            id: format!("raycast:{}", extension.name),
            name: extension.name.clone(),
            path: extension.path.clone(),
            description: extension.description.clone(),
            keywords: extension.categories.clone(), // Use categories as keywords
            version: "1.0.0".to_string(),
            commands: vec!["main".to_string()], // Default command for Raycast extensions
            metadata: std::collections::HashMap::new(),
        };

        Ok(Self {
            extension,
            metadata,
        })
    }

    /// Get the plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get a reference to the wrapped extension
    pub fn extension(&self) -> &RaycastExtension {
        &self.extension
    }
}

impl Plugin for RaycastPluginWrapper {
    fn build(&self, app: &mut App) {
        let metadata = self.metadata.clone();
        let extension = self.extension.clone();
        
        // Use startup system to spawn entity instead of direct world spawning
        app.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(RaycastPluginComponent {
                id: metadata.id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                path: metadata.path.clone(),
                commands: extension
                    .commands
                    .iter()
                    .map(|cmd| cmd.name.clone())
                    .collect(),
                extension: extension.clone(),
            });
        });

        // Move SearchIndex operations to startup system to avoid CommandQueue issues
        let metadata_for_search = self.metadata.clone();
        let extension_for_search = self.extension.clone();
        
        app.add_systems(Startup, move |mut search_index: ResMut<SearchIndex>| {
            // Add main extension entry
            let extension_item = SearchItem::new(
                format!("raycast:{}", metadata_for_search.name),
                metadata_for_search.name.clone(),
                metadata_for_search.description.clone(),
                SearchItemType::Plugin,
            )
            .with_keywords(metadata_for_search.keywords.clone());

            search_index.add_item(extension_item);

            // Add command entries
            for command in &extension_for_search.commands {
                let command_item = SearchItem::new(
                    format!("raycast:{}:{}", metadata_for_search.name, command.name),
                    command.title.clone(),
                    command.description.clone().unwrap_or_default(),
                    SearchItemType::ActionItem,
                );

                search_index.add_item(command_item);
            }

            info!(
                "Added Raycast extension '{}' with {} commands to search index",
                metadata_for_search.name,
                extension_for_search.commands.len()
            );
        });

        info!(
            "Registered Raycast extension: {} from {:?}",
            self.metadata.name, self.metadata.path
        );
    }
}

/// Component that holds Raycast extension instance and metadata
#[derive(Component)]
pub struct RaycastPluginComponent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub commands: Vec<String>,
    pub extension: RaycastExtension,
}

impl RaycastPluginComponent {
    /// Execute a command on this Raycast plugin
    pub fn execute_command(&self, command: &str, _args: &[String]) -> Result<String> {
        // Find the command in the extension
        if let Some(_cmd) = self.extension.commands.iter().find(|c| c.name == command) {
            // Create Deno runtime for execution
            let config = RuntimeConfig::default();
            let channels = RuntimeChannels::default();

            let mut runtime = DenoRuntime::new(config, channels).map_err(|e| {
                crate::Error::PluginError(format!("Failed to create runtime: {}", e))
            })?;

            // Load extension source files
            let source_path = self.path.join("src").join("index.ts");
            let source_code = if source_path.exists() {
                fs::read_to_string(&source_path)
                    .map_err(|e| crate::Error::IoError(format!("Failed to read source: {}", e)))?
            } else {
                // Extension source files don't exist - this is an error
                return Err(crate::Error::PluginError(format!(
                    "Extension '{}' source files not found. Expected '{}'",
                    self.name,
                    source_path.display()
                )));
            };

            // Note: This is a synchronous interface for compatibility
            // In a full Bevy implementation, this would use AsyncComputeTaskPool and
            // component-based execution For now, we use the Deno runtime's blocking
            // execution capability
            let plugin_id = format!("raycast_{}", self.name);

            // Create a simple async runtime for this execution
            // This maintains synchronous interface compatibility while using proper async execution
            // internally
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                crate::Error::PluginError(format!("Failed to create runtime: {}", e))
            })?;

            rt.block_on(async { runtime.execute_plugin(&plugin_id, &source_code).await })
                .map_err(|e| {
                    crate::Error::PluginError(format!("Plugin execution failed: {}", e))
                })?;

            Ok(format!(
                "Successfully executed '{}' on Raycast plugin '{}'",
                command, self.name
            ))
        } else {
            Err(crate::Error::PluginError(format!(
                "Command '{}' not found in plugin '{}'",
                command, self.name
            )))
        }
    }
}

/// System to handle Raycast extension execution requests
pub fn execute_raycast_plugin_system(
    plugins: Query<&RaycastPluginComponent>,
    // Add event readers for plugin execution when available
) {
    // This system will handle execution requests for Raycast extensions
    // Implementation will depend on the event system design
    for plugin_component in &plugins {
        // Extension execution logic will be added based on event system
        debug!(
            "Raycast extension available: {} at {:?}",
            plugin_component.name, plugin_component.path
        );
    }
}
