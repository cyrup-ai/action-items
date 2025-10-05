//! Plugin discovery for Deno plugins
//!
//! This module contains the plugin discovery logic for JavaScript/TypeScript plugins.

use std::path::PathBuf;

use bevy::prelude::*;
use log::{error, info, warn};

use crate::plugins::core::PluginMetadata;
use crate::runtime::deno::plugin_manager::PluginManager;
use crate::runtime::deno::types::RuntimeConfig;
use crate::runtime::plugin_wrapper::core::DenoPluginWrapper;
use crate::search::{SearchIndex, SearchItem, SearchItemType};

/// Task component for async Deno plugin loading
#[derive(Component)]
pub struct DenoPluginLoadingTask {
    pub plugin_paths: Vec<PathBuf>,
    pub loaded_count: usize,
    pub failed_count: usize,
    pub plugin_manager: PluginManager,
}

/// Discovery progress tracking
#[derive(Resource, Default)]
pub struct DenoDiscoveryProgress {
    pub total_discovered: usize,
    pub successfully_loaded: usize,
    pub failed_to_load: usize,
    pub discovery_complete: bool,
}

/// System to start async Deno plugin loading
pub fn start_deno_plugin_loading(mut commands: Commands, config: Res<RuntimeConfig>) {
    info!("Starting Deno plugin discovery and loading...");

    // Discover JavaScript/TypeScript plugin files
    let plugin_paths = discover_deno_plugins();
    info!("Found {} potential Deno plugins", plugin_paths.len());

    // Create plugin manager for this discovery session
    let plugin_manager = PluginManager::new(config.clone());

    // Create loading task component
    commands.spawn(DenoPluginLoadingTask {
        plugin_paths,
        loaded_count: 0,
        failed_count: 0,
        plugin_manager,
    });
}

/// System to handle Deno plugin loading progress
pub fn handle_deno_plugin_loading_system(
    mut commands: Commands,
    mut loading_tasks: Query<(Entity, &mut DenoPluginLoadingTask)>,
    mut search_index: Option<ResMut<SearchIndex>>,
    mut discovery_progress: ResMut<DenoDiscoveryProgress>,
) {
    for (entity, mut task) in &mut loading_tasks {
        // Process one plugin per frame to avoid blocking
        let current_index = task.loaded_count + task.failed_count;
        if let Some(plugin_path) = task.plugin_paths.get(current_index) {
            let plugin_path = plugin_path.clone();
            // Attempt to load plugin using plugin manager
            match task.plugin_manager.load_plugin(&plugin_path) {
                Ok(plugin_id) => {
                    info!(
                        "Successfully loaded Deno plugin: {} from {:?}",
                        plugin_id, plugin_path
                    );

                    // Create DenoPluginWrapper and component
                    if let Ok(plugin_manifest) = task.plugin_manager.load_manifest(&plugin_path) {
                        let metadata = PluginMetadata {
                            id: plugin_id.to_string(),
                            name: plugin_manifest.name.clone(),
                            path: plugin_path.clone(),
                            manifest: plugin_manifest.clone(),
                            is_loaded: true,
                            last_accessed: Some(std::time::SystemTime::now()),
                            load_count: 1,
                        };

                        match DenoPluginWrapper::new(plugin_id.clone(), metadata.clone()) {
                            Ok(_wrapper) => {
                                // Spawn DenoPluginComponent entity
                                commands.spawn(crate::runtime::plugin_wrapper::plugin_component::DenoPluginComponent {
                                    plugin_id: plugin_id.clone(),
                                    name: metadata.name.clone(),
                                    version: plugin_manifest.version.clone(),
                                    description: plugin_manifest.description.clone(),
                                    entry_point: plugin_path.clone(),
                                });

                                // Add to search index if available
                                if let Some(ref mut search_index) = search_index {
                                    let plugin_item = SearchItem::new(
                                        format!("deno:{}", metadata.name),
                                        metadata.name.clone(),
                                        plugin_manifest.description.clone(),
                                        SearchItemType::Plugin,
                                    );
                                    search_index.add_item(plugin_item);
                                }

                                task.loaded_count += 1;
                                discovery_progress.successfully_loaded += 1;
                            },
                            Err(e) => {
                                error!("Failed to create wrapper for plugin {}: {}", plugin_id, e);
                                task.failed_count += 1;
                                discovery_progress.failed_to_load += 1;
                            },
                        }
                    } else {
                        error!("Failed to load manifest for plugin at {:?}", plugin_path);
                        task.failed_count += 1;
                        discovery_progress.failed_to_load += 1;
                    }
                },
                Err(e) => {
                    warn!("Failed to load Deno plugin from {:?}: {}", plugin_path, e);
                    task.failed_count += 1;
                    discovery_progress.failed_to_load += 1;
                },
            }
        }

        // Check if loading is complete
        if task.loaded_count + task.failed_count >= task.plugin_paths.len() {
            info!(
                "Deno plugin loading complete: {} loaded, {} failed",
                task.loaded_count, task.failed_count
            );
            discovery_progress.discovery_complete = true;
            discovery_progress.total_discovered = task.plugin_paths.len();
            commands.entity(entity).despawn();
        }
    }
}

/// Discover JavaScript/TypeScript plugin files
fn discover_deno_plugins() -> Vec<PathBuf> {
    let mut plugin_paths = Vec::new();

    // Plugin directories to search
    let mut search_dirs = vec![
        PathBuf::from("./plugins/deno"),
        PathBuf::from("./plugins/js"),
        PathBuf::from("./plugins/ts"),
    ];

    // Add user-specific directories
    if let Some(config_dir) = dirs::config_dir() {
        search_dirs.extend([
            config_dir.join("Action Items/plugins/deno"),
            config_dir.join("Action Items/plugins/js"),
            config_dir.join("Action Items/plugins/ts"),
        ]);
    }

    if let Some(data_dir) = dirs::data_local_dir() {
        search_dirs.extend([
            data_dir.join("Action Items/plugins/deno"),
            data_dir.join("Action Items/plugins/js"),
            data_dir.join("Action Items/plugins/ts"),
        ]);
    }

    // Platform-specific system directories
    #[cfg(target_os = "macos")]
    {
        search_dirs.extend([
            PathBuf::from("/Applications/Action Items.app/Contents/Resources/plugins/deno"),
            PathBuf::from("/Applications/Action Items.app/Contents/Resources/plugins/js"),
            PathBuf::from("/Applications/Action Items.app/Contents/Resources/plugins/ts"),
        ]);
        if let Some(home) = dirs::home_dir() {
            search_dirs.extend([
                home.join("Library/Application Support/Action Items/plugins/deno"),
                home.join("Library/Application Support/Action Items/plugins/js"),
                home.join("Library/Application Support/Action Items/plugins/ts"),
            ]);
        }
    }

    #[cfg(target_os = "linux")]
    {
        search_dirs.extend([
            PathBuf::from("/usr/share/action-items/plugins/deno"),
            PathBuf::from("/usr/share/action-items/plugins/js"),
            PathBuf::from("/usr/share/action-items/plugins/ts"),
            PathBuf::from("/usr/local/share/action-items/plugins/deno"),
            PathBuf::from("/usr/local/share/action-items/plugins/js"),
            PathBuf::from("/usr/local/share/action-items/plugins/ts"),
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            let base = PathBuf::from(program_files).join("Action Items/plugins");
            search_dirs.extend([base.join("deno"), base.join("js"), base.join("ts")]);
        }
        if let Some(app_data) = dirs::data_dir() {
            let base = app_data.join("Action Items/plugins");
            search_dirs.extend([base.join("deno"), base.join("js"), base.join("ts")]);
        }
    }

    for search_dir in search_dirs {
        if !search_dir.exists() {
            continue;
        }

        // Recursively search for .js, .ts, and .mjs files
        if let Ok(entries) = std::fs::read_dir(&search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(extension) = path.extension().and_then(|ext| ext.to_str())
                        && matches!(extension, "js" | "ts" | "mjs")
                    {
                        plugin_paths.push(path);
                    }
                } else if path.is_dir() {
                    // Check for package.json indicating a Node.js/Deno project
                    let package_json = path.join("package.json");
                    if package_json.exists() {
                        // Look for main entry point
                        if let Ok(content) = std::fs::read_to_string(&package_json)
                            && let Ok(package) = serde_json::from_str::<serde_json::Value>(&content)
                        {
                            let main_file = package
                                .get("main")
                                .and_then(|m| m.as_str())
                                .unwrap_or("index.js");

                            let main_path = path.join(main_file);
                            if main_path.exists() {
                                plugin_paths.push(main_path);
                            }
                        }
                    }

                    // Also check for plugin.toml in directories
                    let plugin_toml = path.join("plugin.toml");
                    if plugin_toml.exists() {
                        plugin_paths.push(path); // Directory with plugin.toml
                    }
                }
            }
        }
    }

    info!(
        "Discovered {} potential Deno plugin files",
        plugin_paths.len()
    );
    plugin_paths
}
