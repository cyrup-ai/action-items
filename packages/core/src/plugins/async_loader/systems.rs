use std::path::PathBuf;

use bevy::ecs::system::Commands;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use ecs_service_bridge::systems::plugin_management::registration::PluginRegistrationQueue;
use log::{error, info};

use super::context::{
    create_deno_plugin_context, create_plugin_context, create_raycast_plugin_context,
};
use super::events::PluginLoadingStarted;
use super::progress::PluginLoadingProgress;
use super::tasks::{LoadingPlugin, PluginLoadingTask};
use crate::discovery::{DiscoveredPlugin, discover_plugin_wrappers};
use crate::plugins::service_bridge_integration::registration::register_plugin_with_service_bridge;

/// System to start async plugin loading
pub fn start_async_plugin_loading(
    mut commands: Commands,
    mut progress: ResMut<PluginLoadingProgress>,
    mut loading_started_events: EventWriter<PluginLoadingStarted>,
    _registration_queue: Res<PluginRegistrationQueue>,
    app_directories: Res<crate::config::AppDirectories>,
) {
    // Only start loading once
    if progress.total_plugins > 0 {
        return;
    }

    info!("Starting async plugin loading...");

    let thread_pool = AsyncComputeTaskPool::get();

    // Discover plugins synchronously first
    let discovered_plugins = discover_plugin_wrappers();
    let plugin_count = discovered_plugins.len();

    if plugin_count == 0 {
        info!("No plugins found to load");
        return;
    }

    // Update progress resource
    *progress = PluginLoadingProgress::new(plugin_count);

    // Send loading started event
    loading_started_events.write(PluginLoadingStarted {
        total_plugins: plugin_count,
    });

    info!("Found {} plugins to load asynchronously", plugin_count);

    // Create loading tasks for each discovered plugin
    for plugin in discovered_plugins {
        let entity = commands.spawn_empty().id();
        let plugin_info = LoadingPlugin {
            path: match &plugin {
                DiscoveredPlugin::Native(w) => PathBuf::from(format!("native:{}", w.metadata().id)),
                DiscoveredPlugin::Extism(w) => PathBuf::from(format!("extism:{}", w.metadata().id)),
                DiscoveredPlugin::Raycast(w) => w.metadata().path.clone(),
                DiscoveredPlugin::Deno(w) => PathBuf::from(format!("deno:{}", w.metadata().id)),
            },
            plugin_type: match &plugin {
                DiscoveredPlugin::Native(_) => "native".to_string(),
                DiscoveredPlugin::Extism(_) => "extism".to_string(),
                DiscoveredPlugin::Raycast(_) => "raycast".to_string(),
                DiscoveredPlugin::Deno(_) => "deno".to_string(),
            },
            plugin_data: plugin.clone(),
        };

        // Create async loading task for this plugin
        let plugin_clone = plugin.clone();
        let entity_id = entity;
        let app_directories_clone = (*app_directories).clone();
        let loading_task = thread_pool.spawn(async move {
            create_plugin_loading_task(plugin_clone, entity_id, app_directories_clone).await
        });

        commands
            .entity(entity)
            .insert((plugin_info, PluginLoadingTask(loading_task)));
    }
}

/// Creates the async plugin loading task logic
async fn create_plugin_loading_task(
    plugin_clone: DiscoveredPlugin,
    entity_id: Entity,
    app_directories: crate::config::AppDirectories,
) -> CommandQueue {
    // Perform actual plugin initialization work
    futures_lite::future::yield_now().await;

    let mut command_queue = CommandQueue::default();

    // Initialize plugin based on type - properly handle async initialization
    let initialization_result: Result<(), String> = match &plugin_clone {
        DiscoveredPlugin::Native(wrapper) => {
            initialize_native_plugin(wrapper, &app_directories).await
        },
        DiscoveredPlugin::Extism(wrapper) => {
            initialize_extism_plugin(wrapper, &app_directories).await
        },
        DiscoveredPlugin::Raycast(wrapper) => {
            initialize_raycast_plugin(wrapper, &app_directories).await
        },
        DiscoveredPlugin::Deno(wrapper) => initialize_deno_plugin(wrapper, &app_directories).await,
    };

    // Queue commands based on initialization result
    command_queue.push(move |world: &mut World| {
        handle_initialization_result(world, initialization_result, plugin_clone, entity_id);
    });

    command_queue
}

/// Initialize a native plugin
async fn initialize_native_plugin(
    wrapper: &crate::native_plugin_wrapper::NativePluginWrapper,
    app_directories: &crate::config::AppDirectories,
) -> Result<(), String> {
    // Create context for initialization
    let context = create_plugin_context(&wrapper.metadata().manifest, app_directories)?;
    let task_pool = AsyncComputeTaskPool::get();

    // Create initialization task within a scoped block to ensure guard is dropped
    let init_task = {
        let plugin_arc = wrapper.plugin();
        let mut plugin_guard = plugin_arc.write();
        plugin_guard.initialize(context, task_pool)
    };

    // Await the initialization task
    init_task
        .await
        .map_err(|e| format!("Native plugin '{}': {}", wrapper.metadata().id, e))
}

/// Initialize an Extism plugin
async fn initialize_extism_plugin(
    wrapper: &crate::extism_plugin_wrapper::ExtismPluginWrapper,
    app_directories: &crate::config::AppDirectories,
) -> Result<(), String> {
    // Create context for initialization
    let context = create_plugin_context(&wrapper.metadata().manifest, app_directories)?;
    let task_pool = AsyncComputeTaskPool::get();

    // Create initialization task within a scoped block to ensure guard is dropped
    let init_task = {
        let adapter_arc = wrapper.adapter();
        let mut adapter_guard = adapter_arc.write();
        adapter_guard.initialize(context, task_pool)
    };

    // Await the initialization task
    init_task
        .await
        .map_err(|e| format!("Extism plugin '{}': {}", wrapper.metadata().id, e))
}

/// Initialize a Raycast plugin via Deno runtime
async fn initialize_raycast_plugin(
    wrapper: &crate::raycast::wrapper::RaycastPluginWrapper,
    app_directories: &crate::config::AppDirectories,
) -> Result<(), String> {
    // Initialize Deno runtime for Raycast extension
    // Raycast extensions run on Deno runtime, not as native plugins
    info!("Initializing Raycast extension: {}", wrapper.metadata().id);

    // Create context for Raycast plugin initialization
    let _context = create_raycast_plugin_context(wrapper.metadata(), app_directories)
        .map_err(|e| format!("Failed to create Raycast plugin context: {}", e))?;

    // Deno runtime initialization will be handled when the runtime system is fully implemented
    info!(
        "Created context for Raycast extension: {}",
        wrapper.metadata().name
    );

    Ok(())
}

/// Initialize a Deno plugin asynchronously
async fn initialize_deno_plugin(
    wrapper: &crate::runtime::plugin_wrapper::core::DenoPluginWrapper,
    app_directories: &crate::config::AppDirectories,
) -> Result<(), String> {
    // Initialize Deno plugin with runtime integration
    info!("Initializing Deno plugin: {}", wrapper.metadata().id);

    // Create context for Deno plugin initialization
    let _context = create_deno_plugin_context(wrapper.metadata(), app_directories)
        .map_err(|e| format!("Failed to create Deno plugin context: {}", e))?;

    // Deno runtime initialization will be handled by the runtime system
    info!(
        "Created context for Deno plugin: {}",
        wrapper.metadata().name
    );

    Ok(())
}

/// Handle the result of plugin initialization
fn handle_initialization_result(
    world: &mut World,
    initialization_result: Result<(), String>,
    plugin_clone: DiscoveredPlugin,
    entity_id: Entity,
) {
    match initialization_result {
        Ok(_) => {
            // Plugin initialized successfully - register with service bridge
            if let Some(registration_queue) = world.get_resource::<PluginRegistrationQueue>() {
                match register_plugin_with_service_bridge(registration_queue, &plugin_clone) {
                    Ok(registration) => {
                        info!("Successfully registered plugin with service bridge");

                        // Add service bridge registration to entity
                        world
                            .entity_mut(entity_id)
                            .insert(registration)
                            .remove::<PluginLoadingTask>();
                    },
                    Err(e) => {
                        error!("Failed to register plugin with service bridge: {}", e);
                        world.entity_mut(entity_id).remove::<PluginLoadingTask>();
                    },
                }
            }

            // Update loading progress
            if let Some(mut progress) = world.get_resource_mut::<PluginLoadingProgress>() {
                progress.mark_loaded();
            }
        },
        Err(error_msg) => {
            error!("Failed to initialize plugin: {}", error_msg);

            // Update loading progress
            if let Some(mut progress) = world.get_resource_mut::<PluginLoadingProgress>() {
                progress.mark_failed();
            }

            // Remove failed plugin entity
            world.entity_mut(entity_id).remove::<PluginLoadingTask>();
        },
    }
}
