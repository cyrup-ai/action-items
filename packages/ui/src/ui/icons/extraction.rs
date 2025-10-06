use std::path::{Path, PathBuf};

// Note: CoreActionItem will be used when icon extraction is implemented
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures};
use tracing::warn;

use crate::ui::icons::types::LauncherIconCache;
use action_items_ecs_ui::icons::{IconExtractionRequest, IconExtractionResult, IconSize};

#[derive(Component)]
pub struct IconExtractionInProgress(Task<CommandQueue>);

pub fn request_icon_extraction(
    events: &mut EventWriter<IconExtractionRequest>,
    result: &action_items_core::plugins::ActionItem,
    size: IconSize,
) {
    let path_str_option: Option<&str> = if let Some(icon_str) = &result.icon {
        Some(icon_str.as_str())
    } else if result.action.starts_with("app::") {
        result.action.strip_prefix("app::")
    } else if result.action.starts_with("file::") {
        result.action.strip_prefix("file::")
    } else {
        None
    };

    let path_str = match path_str_option {
        Some(path) => path,
        None => {
            // debug!("No suitable path for icon extraction from SearchResult: {:?}", result);
            return;
        },
    };

    let path = PathBuf::from(path_str);

    if path_str.is_empty()
        || (!path.is_file()
            && path.extension().is_none()
            && !path_str.contains('/')
            && !path_str.starts_with("id::"))
    {
        // debug!("Skipping icon extraction for non-file-like or empty path: {}", path_str);
        return;
    }

    let request = IconExtractionRequest {
        id: result.action.clone(),
        path,
        size,
    };

    // debug!("Requesting icon extraction: {:?}", request);
    events.write(request);
}

pub fn process_icon_extraction_requests(
    mut commands: Commands,
    mut events: EventReader<IconExtractionRequest>,
) {
    for request in events.read() {
        let req_id = request.id.clone();
        let req_path = request.path.clone();
        let req_size_pixels = request.size.pixels();

        // Spawn an entity to own the task
        let entity = commands.spawn_empty().id();

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let result: Option<(Vec<u8>, u32, u32)> =
                extract_icon_from_file(&req_path, req_size_pixels);

            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| {
                // Use SystemState to safely access multiple resources - following
                // bevy/examples/async_tasks/async_compute.rs pattern
                let (_icon_cache_handle, _images_handle) = {
                    let mut system_state =
                        SystemState::<(ResMut<LauncherIconCache>, ResMut<Assets<Image>>)>::new(world);
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
                            icon_cache.loaded_icons_mut().insert(req_id.clone(), handle);
                            icon_cache.failed_to_load_mut().remove(&req_id);
                            // debug!("Icon extracted and cached for id: {}", req_id);
                        },
                        None => {
                            // debug!("Icon extraction failed for id (in task callback): {}",
                            // req_id);
                            icon_cache.failed_to_load_mut().insert(req_id.clone());
                        },
                    }

                    ((), ()) // Return fallback values since we're done with the resources
                };

                // Despawn the entity that held the task
                world.despawn(entity);
            });
            command_queue
        });
        commands
            .entity(entity)
            .insert(IconExtractionInProgress(task));
    }
}

pub fn poll_icon_extraction_tasks(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut IconExtractionInProgress)>,
    _result_events: EventWriter<IconExtractionResult>,
    _icon_cache: ResMut<LauncherIconCache>,
) {
    for (entity, mut in_progress_task) in task_query.iter_mut() {
        match futures::check_ready(&mut in_progress_task.0) {
            Some(mut command_queue) => {
                // The task now returns CommandQueue
                // Apply the command queue to the world
                commands.append(&mut command_queue);
                // The task entity is despawned within the CommandQueue closure, so no need to
                // despawn here.
            },
            None => {
                // Task is not yet complete, do nothing or check if finished for cleanup
                if in_progress_task.0.is_finished() {
                    warn!(
                        "Icon extraction task finished but poll_once returned None and no \
                         CommandQueue. Despawning task entity: {:?}",
                        entity
                    );
                    commands.entity(entity).despawn();
                }
            },
        }
    }
}

pub fn process_icon_extraction_results(
    mut events: EventReader<IconExtractionResult>,
    mut icon_cache: ResMut<LauncherIconCache>,
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
        icon_cache.loaded_icons_mut().insert(result.id.clone(), handle);
    }
}

fn extract_icon_from_file(path: &Path, _size: u32) -> Option<(Vec<u8>, u32, u32)> {
    #[cfg(target_os = "windows")]
    {
        extract_windows_icon(path, _size)
    }
    #[cfg(target_os = "macos")]
    {
        extract_macos_icon(path, _size)
    }
    #[cfg(target_os = "linux")]
    {
        extract_linux_icon(path, _size)
    }
}

#[cfg(target_os = "windows")]
fn extract_windows_icon(_path: &PathBuf, _size: u32) -> Option<(Vec<u8>, u32, u32)> {
    // Windows icon extraction would go here
    // For now, return None to use fallback
    None
}

#[cfg(target_os = "macos")]
fn extract_macos_icon(_path: &Path, _size: u32) -> Option<(Vec<u8>, u32, u32)> {
    // macOS icon extraction would go here
    // For now, return None to use fallback
    None
}

#[cfg(target_os = "linux")]
fn extract_linux_icon(_path: &PathBuf, _size: u32) -> Option<(Vec<u8>, u32, u32)> {
    // Linux icon extraction would go here
    // For now, return None to use fallback
    None
}
