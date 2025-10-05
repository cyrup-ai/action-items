use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};

use crate::icons::{IconCache, IconExtractionRequest, IconExtractionResult};
use super::platform::extract_icon_from_file;

/// Component marking entity with ongoing icon extraction task
///
/// Wraps a Bevy async task that returns a CommandQueue for deferred world access.
/// The task performs platform-specific I/O on background thread pool.
#[derive(Component)]
pub struct IconExtractionInProgress(pub Task<CommandQueue>);

/// Process icon extraction requests
///
/// Spawns async tasks to extract icons from file paths.
/// Each task runs on AsyncComputeTaskPool to avoid blocking main thread.
///
/// # Workflow
/// 1. Read IconExtractionRequest events
/// 2. For each request, spawn background task
/// 3. Task extracts platform icon to RGBA data
/// 4. Task builds CommandQueue to update IconCache
/// 5. Attach task to entity via IconExtractionInProgress
///
/// # Pattern
/// Follows canonical Bevy async pattern from bevy/examples/async_tasks/async_compute.rs
pub fn process_icon_extraction_requests(
    mut commands: Commands,
    mut events: EventReader<IconExtractionRequest>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    
    for request in events.read() {
        let req_id = request.id.clone();
        let req_path = request.path.clone();
        let req_size_pixels = request.size.pixels();

        // Spawn entity to own the task
        let entity = commands.spawn_empty().id();

        let task = thread_pool.spawn(async move {
            let result: Option<(Vec<u8>, u32, u32)> =
                extract_icon_from_file(&req_path, req_size_pixels);

            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| {
                // Use SystemState for safe resource access from async context
                let mut system_state =
                    SystemState::<(ResMut<IconCache>, ResMut<Assets<Image>>)>::new(world);
                let (mut icon_cache, mut images) = system_state.get_mut(world);

                match result {
                    Some((data, width, height)) => {
                        let image = Image::new(
                            bevy::render::render_resource::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                            bevy::render::render_resource::TextureDimension::D2,
                            data,
                            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                            RenderAssetUsages::RENDER_WORLD,
                        );
                        let handle = images.add(image);
                        icon_cache.loaded_icons.insert(req_id.clone(), handle);
                        icon_cache.failed_to_load.remove(&req_id);
                    },
                    None => {
                        icon_cache.failed_to_load.insert(req_id.clone());
                    },
                }

                // Despawn task entity now that work is complete
                world.despawn(entity);
            });
            command_queue
        });
        
        commands.entity(entity).insert(IconExtractionInProgress(task));
    }
}

/// Poll ongoing icon extraction tasks
///
/// Checks async tasks for completion and applies results via CommandQueue.
///
/// # Workflow
/// 1. Query all entities with IconExtractionInProgress
/// 2. Poll each task for completion
/// 3. If complete, append CommandQueue to apply world changes
/// 4. Task's CommandQueue handles cache update and entity cleanup
///
/// # Pattern
/// Uses canonical `block_on(future::poll_once())` pattern from Bevy examples
pub fn poll_icon_extraction_tasks(
    mut commands: Commands,
    mut task_query: Query<&mut IconExtractionInProgress>,
) {
    for mut in_progress_task in task_query.iter_mut() {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut in_progress_task.0)) {
            // Task complete, apply deferred commands
            commands.append(&mut command_queue);
        }
    }
}

/// Process icon extraction results
///
/// Handles IconExtractionResult events and updates IconCache.
/// This provides alternative event-based workflow to CommandQueue pattern.
///
/// # Note
/// Currently the systems use CommandQueue pattern (process_icon_extraction_requests
/// updates cache directly). This system provides compatibility for external code
/// that may send IconExtractionResult events.
pub fn process_icon_extraction_results(
    mut events: EventReader<IconExtractionResult>,
    mut icon_cache: ResMut<IconCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for result in events.read() {
        let image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: result.width,
                height: result.height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            result.icon_data.clone(),
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );

        let handle = images.add(image);
        icon_cache.loaded_icons.insert(result.id.clone(), handle);
    }
}
