use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::*;
use log::{error, info};

use crate::raycast::adapter::RaycastAdapter;
use crate::raycast::loader::RaycastLoader;
use crate::raycast::wrapper::{RaycastPluginComponent, RaycastPluginWrapper};
use crate::search::{SearchIndex, SearchItem, SearchItemType};

/// Resource to hold the Raycast loader
#[derive(Resource)]
pub struct RaycastManager {
    loader: RaycastLoader,
    adapter: Arc<RaycastAdapter>,
    initialized: bool,
}

impl RaycastManager {
    pub fn new(config_dir: PathBuf) -> Self {
        let loader = RaycastLoader::new(&config_dir);
        let runtime_path = config_dir.join("raycast-runtime");
        let adapter = Arc::new(RaycastAdapter::new(runtime_path));

        Self {
            loader,
            adapter,
            initialized: false,
        }
    }
}

/// System to discover and load Raycast extensions using plugin wrappers
pub fn discover_raycast_extensions(
    mut raycast_manager: ResMut<RaycastManager>,
    mut commands: Commands,
    mut search_index: Option<ResMut<SearchIndex>>,
) {
    // Initialize Raycast loader (clone repo if needed)
    if !raycast_manager.initialized {
        match raycast_manager.loader.initialize() {
            Ok(_) => {
                // API shim creation is handled by the adapter internally
                info!("Raycast extensions initialized successfully");
                raycast_manager.initialized = true;
            },
            Err(e) => {
                error!("Failed to initialize Raycast extensions: {}", e);
                return;
            },
        }
    }

    // List all available Raycast extensions
    match raycast_manager.loader.list_extensions() {
        Ok(extensions) => {
            info!("Found {} Raycast extensions", extensions.len());

            for extension in extensions {
                // Convert extension using adapter for proper manifest format
                let _plugin_manifest = raycast_manager.adapter.to_plugin_manifest(&extension);

                // Create a RaycastPluginWrapper for each extension
                match RaycastPluginWrapper::new(extension.clone()) {
                    Ok(wrapper) => {
                        // Manually perform plugin registration since we can't add plugins at
                        // runtime

                        // 1. Spawn RaycastPluginComponent entity
                        commands.spawn(RaycastPluginComponent {
                            id: wrapper.metadata().id.clone(),
                            name: wrapper.metadata().name.clone(),
                            description: wrapper.metadata().description.clone(),
                            path: wrapper.metadata().path.clone(),
                            commands: wrapper.metadata().commands.clone(),
                            extension: extension.clone(),
                        });

                        // 2. Add items to SearchIndex if available
                        if let Some(ref mut search_index) = search_index {
                            // Add main extension entry
                            let extension_item = SearchItem::new(
                                wrapper.metadata().id.clone(),
                                wrapper.metadata().name.clone(),
                                wrapper.metadata().description.clone(),
                                SearchItemType::Plugin,
                            )
                            .with_keywords(wrapper.metadata().keywords.clone());

                            search_index.add_item(extension_item);

                            // Add individual commands from the extension
                            for command in &extension.commands {
                                let command_item = SearchItem::new(
                                    format!(
                                        "raycast:{}:command:{}",
                                        wrapper.metadata().name,
                                        command.name
                                    ),
                                    command.title.clone(),
                                    command
                                        .description
                                        .clone()
                                        .unwrap_or_else(|| command.name.clone()),
                                    SearchItemType::Plugin,
                                )
                                .with_keywords(wrapper.metadata().keywords.clone());

                                search_index.add_item(command_item);
                            }
                        }

                        info!(
                            "Registered Raycast extension: {} from {:?}",
                            wrapper.metadata().name,
                            wrapper.metadata().path
                        );
                    },
                    Err(e) => {
                        error!(
                            "Failed to create wrapper for Raycast extension {}: {}",
                            extension.id, e
                        );
                    },
                }
            }
        },
        Err(e) => {
            error!("Failed to list Raycast extensions: {}", e);
        },
    }
}

/// System to periodically check for Raycast extension updates
pub fn update_raycast_extensions(
    raycast_manager: Res<RaycastManager>,
    time: Res<Time>,
    mut last_check: Local<f32>,
) {
    // Check every hour if we should update (24hr update cycle handled by loader)
    let current_time = time.elapsed().as_secs_f32();
    if current_time - *last_check > 3600.0 {
        *last_check = current_time;

        // The loader will check if 24 hours have passed since last sync
        if let Err(e) = raycast_manager.loader.initialize() {
            error!("Failed to update Raycast extensions: {}", e);
        }
    }
}
