use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};
use tokio::sync::mpsc;

use crate::events::{
    FileSystemChanged, FileSystemOperationFailed, FileSystemRequest, FileSystemResponse,
};
use crate::manager::FileSystemManager;
use crate::security::SecurityConfig;
use crate::types::FileOperationId;
use crate::watcher::FileSystemWatcher;

/// Bevy resource for filesystem configuration
#[derive(Resource)]
pub struct FileSystemConfig {
    pub security: SecurityConfig,
    pub enable_watching: bool,
    pub cache_ttl: Duration,
    pub max_concurrent_operations: usize,
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            security: SecurityConfig::default(),
            enable_watching: true,
            cache_ttl: Duration::from_secs(30),
            max_concurrent_operations: 10,
        }
    }
}

/// Bevy resource for filesystem manager
#[derive(Resource)]
pub struct FileSystemResource {
    manager: Arc<FileSystemManager>,
    watcher: Option<Arc<FileSystemWatcher>>,
    watch_event_receiver: Option<mpsc::UnboundedReceiver<Vec<crate::types::FileSystemChange>>>,
}

/// Component for async filesystem tasks following async_compute.rs pattern
#[derive(Component)]
pub struct FileSystemTask(pub Task<FileSystemResponse>);

/// Component for filesystem watch tasks
#[derive(Component)]
pub struct FileSystemWatchTask {
    pub operation_id: FileOperationId,
    pub path: std::path::PathBuf,
}

/// Bevy plugin for filesystem operations
pub struct FileSystemPlugin;

impl Plugin for FileSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileSystemConfig>()
            .add_event::<FileSystemRequest>()
            .add_event::<FileSystemResponse>()
            .add_event::<FileSystemChanged>()
            .add_event::<FileSystemOperationFailed>()
            .add_systems(Startup, initialize_filesystem_service)
            .add_systems(
                Update,
                (
                    handle_filesystem_requests,
                    handle_filesystem_tasks,
                    handle_watch_events,
                    cleanup_filesystem_cache,
                )
                    .chain(),
            );
    }
}

/// Initialize filesystem service on startup
fn initialize_filesystem_service(mut commands: Commands, config: Res<FileSystemConfig>) {
    let manager = Arc::new(FileSystemManager::new(config.security.clone()));

    let (watcher, watch_receiver) = if config.enable_watching {
        let (tx, rx) = mpsc::unbounded_channel();
        let validator = crate::security::PathValidator::new(config.security.clone());
        let watcher = Arc::new(FileSystemWatcher::new(validator, tx));
        (Some(watcher), Some(rx))
    } else {
        (None, None)
    };

    commands.insert_resource(FileSystemResource {
        manager,
        watcher,
        watch_event_receiver: watch_receiver,
    });

    info!("FileSystem ECS service initialized with security and performance optimizations");
}
/// Handle incoming filesystem requests - creates Task components following async_compute.rs pattern
fn handle_filesystem_requests(
    mut commands: Commands,
    mut events: EventReader<FileSystemRequest>,
    fs_resource: Res<FileSystemResource>,
) {
    for event in events.read() {
        let manager = fs_resource.manager.clone();

        let task = match event {
            FileSystemRequest::ReadFile {
                operation_id,
                path,
                requester,
            } => {
                let operation_id = *operation_id;
                let path = path.clone();
                let requester = *requester;

                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.read_file(operation_id, &path).await;
                    FileSystemResponse::ReadFileResult {
                        operation_id,
                        requester,
                        result: Box::new(result),
                    }
                })
            },

            FileSystemRequest::WriteFile {
                operation_id,
                path,
                content,
                requester,
            } => {
                let operation_id = *operation_id;
                let path = path.clone();
                let content = content.clone();
                let requester = *requester;

                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager
                        .write_file(operation_id, &path, &content, true)
                        .await;
                    FileSystemResponse::WriteFileResult {
                        operation_id,
                        requester,
                        result,
                    }
                })
            },

            FileSystemRequest::ListDirectory {
                operation_id,
                path,
                requester,
            } => {
                let operation_id = *operation_id;
                let path = path.clone();
                let requester = *requester;

                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.list_directory(operation_id, &path, false).await;
                    FileSystemResponse::ListDirectoryResult {
                        operation_id,
                        requester,
                        result: Box::new(result),
                    }
                })
            },

            FileSystemRequest::CreateDirectory {
                operation_id,
                path,
                requester,
            } => {
                let operation_id = *operation_id;
                let path = path.clone();
                let requester = *requester;

                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.create_directory(operation_id, &path, true).await;
                    FileSystemResponse::CreateDirectoryResult {
                        operation_id,
                        requester,
                        result,
                    }
                })
            },
            FileSystemRequest::WatchDirectory {
                operation_id,
                path,
                config,
                requester,
            } => {
                if let Some(ref watcher) = fs_resource.watcher {
                    let operation_id = *operation_id;
                    let path = path.clone();
                    let config = *config.clone();
                    let requester = *requester;
                    let watcher = watcher.clone();

                    AsyncComputeTaskPool::get().spawn(async move {
                        let result = watcher.start_watching(operation_id, &path, config);
                        FileSystemResponse::WatchDirectoryResult {
                            operation_id,
                            requester,
                            result,
                        }
                    })
                } else {
                    // Watching disabled, return error immediately
                    let operation_id = *operation_id;
                    let requester = *requester;

                    AsyncComputeTaskPool::get().spawn(async move {
                        FileSystemResponse::WatchDirectoryResult {
                            operation_id,
                            requester,
                            result: Err(crate::types::FileSystemError::ResourceExhausted {
                                resource: "filesystem watching disabled".to_string(),
                            }),
                        }
                    })
                }
            },

            FileSystemRequest::CheckPermissions {
                operation_id,
                path,
                requester,
            } => {
                let operation_id = *operation_id;
                let path = path.clone();
                let requester = *requester;

                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.get_metadata(&path).await;
                    FileSystemResponse::CheckPermissionsResult {
                        operation_id,
                        requester,
                        result,
                    }
                })
            },
        };

        // Spawn entity with Task component following async_compute.rs pattern
        commands.spawn(FileSystemTask(task));
    }
}

/// Handle completed filesystem tasks - following async_compute.rs pattern exactly
fn handle_filesystem_tasks(
    mut commands: Commands,
    mut filesystem_tasks: Query<(Entity, &mut FileSystemTask)>,
    mut response_writer: EventWriter<FileSystemResponse>,
) {
    for (entity, mut task) in &mut filesystem_tasks {
        if let Some(response) = block_on(future::poll_once(&mut task.0)) {
            // Task is complete, send response event and remove task component
            response_writer.write(response);
            commands.entity(entity).despawn();
        }
    }
}
/// Handle filesystem watch events from the background watcher
fn handle_watch_events(
    mut fs_resource: ResMut<FileSystemResource>,
    mut change_events: EventWriter<FileSystemChanged>,
) {
    if let Some(ref mut receiver) = fs_resource.watch_event_receiver {
        // Process all available watch events without blocking
        while let Ok(changes) = receiver.try_recv() {
            if !changes.is_empty() {
                change_events.write(FileSystemChanged { changes });
            }
        }
    }
}

/// Periodically cleanup filesystem cache to prevent memory leaks
fn cleanup_filesystem_cache(
    time: Res<Time>,
    mut last_cleanup: Local<f32>,
    fs_resource: Res<FileSystemResource>,
) {
    const CLEANUP_INTERVAL: f32 = 60.0; // Cleanup every 60 seconds

    *last_cleanup += time.delta_secs();

    if *last_cleanup >= CLEANUP_INTERVAL {
        fs_resource.manager.cleanup_cache();
        *last_cleanup = 0.0;

        trace!("Filesystem cache cleanup completed");
    }
}

// Extension trait for easier filesystem operations from ECS systems
impl FileSystemResource {
    /// Convenience method for synchronous file reading (use sparingly)
    pub fn read_file_sync(
        &self,
        path: &std::path::Path,
    ) -> Result<Vec<u8>, crate::types::FileSystemError> {
        let operation_id = FileOperationId::new();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let content = self.manager.read_file(operation_id, path).await?;
                Ok(content.data)
            })
        })
    }

    /// Convenience method for synchronous file writing (use sparingly)
    pub fn write_file_sync(
        &self,
        path: &std::path::Path,
        content: &[u8],
    ) -> Result<(), crate::types::FileSystemError> {
        let operation_id = FileOperationId::new();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.manager
                    .write_file(operation_id, path, content, true)
                    .await
            })
        })
    }
}
