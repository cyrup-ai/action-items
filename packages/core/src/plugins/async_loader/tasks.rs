use std::path::PathBuf;

use bevy::ecs::system::Commands;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::Task;

use crate::discovery::DiscoveredPlugin;

/// Component that holds an async plugin loading task
#[derive(Component)]
pub struct PluginLoadingTask(pub Task<CommandQueue>);

/// Component marker for entities that are loading plugins
#[derive(Component)]
pub struct LoadingPlugin {
    pub path: PathBuf,
    pub plugin_type: String,
    pub plugin_data: DiscoveredPlugin,
}

/// System to handle completed plugin loading tasks
pub fn handle_plugin_loading_tasks(
    mut commands: Commands,
    mut loading_tasks: Query<(Entity, &mut PluginLoadingTask, &LoadingPlugin)>,
    _app_directories: Res<crate::config::AppDirectories>,
) {
    use bevy::tasks::block_on;
    use bevy::tasks::futures_lite::future;

    for (_entity, mut task, loading_plugin) in &mut loading_tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Task completed - apply the command queue
            log::info!(
                "Plugin loading task completed for: {:?}",
                loading_plugin.path
            );

            // Apply the deferred commands from the async task
            commands.append(&mut command_queue);
        }
    }
}
