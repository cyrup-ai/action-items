use std::time::Duration;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};
use bevy::time::{Timer, TimerMode};
use tracing::{error, info};

use crate::manager::ArboardManager;
use crate::types::{ClipboardData, ClipboardError, ClipboardFormat};

/// Simple Resource for clipboard state - following Bevy ECS patterns
#[derive(Resource)]
pub struct ClipboardResource {
    /// Whether clipboard is available on this platform
    pub available: bool,
    /// Last known clipboard formats (cached for performance)
    pub last_known_formats: Vec<ClipboardFormat>,
    /// Timer for throttling clipboard change detection (checks every 2 seconds)
    pub change_detection_timer: Timer,
}

impl Default for ClipboardResource {
    fn default() -> Self {
        Self {
            available: true, // Will be tested in startup system
            last_known_formats: Vec::new(),
            change_detection_timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
        }
    }
}

impl ClipboardResource {
    /// Get clipboard content synchronously (blocking) - matches ARCHITECTURE.md
    /// WARNING: This blocks the main thread - prefer async ClipboardRequest events
    pub fn get_sync(&self, format: ClipboardFormat) -> Result<ClipboardData, ClipboardError> {
        if !self.available {
            return Err(ClipboardError::AccessDenied);
        }

        // Use block_on with AsyncComputeTaskPool for proper Bevy async integration
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(ArboardManager::get(format));
        block_on(task)
    }

    /// Set clipboard content synchronously (blocking) - matches ARCHITECTURE.md  
    /// WARNING: This blocks the main thread - prefer async ClipboardRequest events
    pub fn set_sync(&self, data: ClipboardData) -> Result<(), ClipboardError> {
        if !self.available {
            return Err(ClipboardError::AccessDenied);
        }

        // Use block_on with AsyncComputeTaskPool for proper Bevy async integration
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(ArboardManager::set(data));
        block_on(task)
    }

    /// Clear clipboard synchronously (blocking) - matches ARCHITECTURE.md
    /// WARNING: This blocks the main thread - prefer async ClipboardRequest events
    pub fn clear_sync(&self) -> Result<(), ClipboardError> {
        if !self.available {
            return Err(ClipboardError::AccessDenied);
        }

        // Use block_on with AsyncComputeTaskPool for proper Bevy async integration
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(ArboardManager::clear());
        block_on(task)
    }

    /// Check if format is available synchronously - matches ARCHITECTURE.md
    /// WARNING: This blocks the main thread - prefer async ClipboardRequest events
    pub fn has_format_sync(&self, format: ClipboardFormat) -> bool {
        if !self.available {
            return false;
        }

        // Use block_on with AsyncComputeTaskPool for proper Bevy async integration
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(ArboardManager::has_format(format));
        block_on(task)
    }

    /// Get available formats synchronously - matches ARCHITECTURE.md
    /// WARNING: This blocks the main thread - prefer async ClipboardRequest events
    pub fn available_formats_sync(&self) -> Vec<ClipboardFormat> {
        if !self.available {
            return vec![];
        }

        // Use block_on with AsyncComputeTaskPool for proper Bevy async integration
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(ArboardManager::available_formats());
        block_on(task)
    }
}

/// Bevy event for clipboard operations - following ARCHITECTURE.md spec
#[derive(Event, Debug)]
pub enum ClipboardRequest {
    Get {
        format: ClipboardFormat,
        requester: Entity,
    },
    Set {
        data: ClipboardData,
        requester: Entity,
    },
    Clear {
        requester: Entity,
    },
    CheckFormat {
        format: ClipboardFormat,
        requester: Entity,
    },
    GetAvailableFormats {
        requester: Entity,
    },
}

/// Bevy event for clipboard responses - sent back to requester entity
#[derive(Event, Debug)]
pub enum ClipboardResponse {
    GetResult {
        requester: Entity,
        result: Result<ClipboardData, ClipboardError>,
    },
    SetResult {
        requester: Entity,
        result: Result<(), ClipboardError>,
    },
    ClearResult {
        requester: Entity,
        result: Result<(), ClipboardError>,
    },
    CheckFormatResult {
        requester: Entity,
        result: bool,
    },
    AvailableFormatsResult {
        requester: Entity,
        result: Vec<ClipboardFormat>,
    },
}

/// Bevy event fired when clipboard content changes - matches ARCHITECTURE.md
#[derive(Event, Debug)]
pub struct ClipboardChanged {
    pub available_formats: Vec<ClipboardFormat>,
}

/// Component for async clipboard tasks - following async_compute.rs pattern
#[derive(Component)]
pub struct ClipboardTask(pub Task<ClipboardResponse>);

/// Component for clipboard availability test task - following async_compute.rs pattern
#[derive(Component)]
pub struct ClipboardAvailabilityTask(pub Task<bool>);

/// Component for clipboard watcher entities with configurable intervals
#[derive(Component)]
pub struct ClipboardWatcher {
    pub last_sequence: u64,
    pub check_interval: Timer,
    pub active: bool,
}

impl Default for ClipboardWatcher {
    fn default() -> Self {
        Self {
            last_sequence: 0,
            check_interval: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
            active: true,
        }
    }
}

/// Component for async clipboard sequence number checking tasks
#[derive(Component)]
pub struct ClipboardCheckTask {
    pub task: Task<Result<u64, ClipboardError>>,
    pub previous_sequence: u64,
    pub requester: Entity,
}

/// Component for re-enabling failed clipboard watchers after delay
#[derive(Component)]
pub struct ClipboardWatcherReenableTimer {
    pub timer: Timer,
}

impl Default for ClipboardWatcherReenableTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
        }
    }
}

/// Get clipboard sequence number for change detection
async fn get_clipboard_sequence() -> Result<u64, ClipboardError> {
    #[cfg(target_os = "macos")]
    {
        // Custom objc2 binding for NSPasteboard changeCount
        use objc2::msg_send;
        use objc2_app_kit::NSPasteboard;
        
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let change_count: i64 = msg_send![&pasteboard, changeCount];
            Ok(change_count as u64)
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
        let sequence = unsafe { GetClipboardSequenceNumber() };
        Ok(sequence as u64)
    }
    
    #[cfg(target_os = "linux")]
    {
        // Hash clipboard content for change detection
        use sha2::{Sha256, Digest};
        
        // Try to get available formats and hash all content for comprehensive change detection
        let formats = ArboardManager::available_formats().await;
        let mut hasher = Sha256::new();
        
        // Hash the available formats first
        for format in &formats {
            hasher.update(format!("{:?}", format).as_bytes());
        }
        
        // Hash text content if available
        if let Ok(text) = ArboardManager::get_text().await {
            hasher.update(text.as_bytes());
        }
        
        // Hash file list if available
        if let Ok(files) = ArboardManager::get_files().await {
            for file in files {
                if let Some(path_str) = file.to_str() {
                    hasher.update(path_str.as_bytes());
                }
            }
        }
        
        let hash = hasher.finalize();
        // Convert first 8 bytes to u64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash[0..8]);
        Ok(u64::from_le_bytes(bytes))
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        // Fallback for unsupported platforms - return error
        Err(ClipboardError::UnsupportedPlatform)
    }
}

/// Bevy plugin for clipboard system
pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClipboardResource>()
            .add_event::<ClipboardRequest>()
            .add_event::<ClipboardResponse>()
            .add_event::<ClipboardChanged>()
            .add_event::<crate::types::ClipboardChangeEvent>() // NEW
            // Systems run in proper order following event.rs chaining pattern
            .add_systems(
                Update,
                (
                    handle_clipboard_requests,
                    handle_clipboard_tasks,
                    handle_clipboard_availability_task,
                    detect_clipboard_changes,        // UPDATED
                    process_clipboard_check_tasks,   // NEW
                    reenable_clipboard_watchers,     // NEW
                )
                    .chain(),
            )
            .add_systems(Startup, (
                spawn_clipboard_availability_test,
                spawn_default_clipboard_watcher,    // NEW
            ));
    }
}

/// Startup system to spawn clipboard availability test - following async_compute.rs pattern
fn spawn_clipboard_availability_test(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        // Test if arboard clipboard is available
        match ArboardManager::get_text().await {
            Ok(_) => true,
            Err(ClipboardError::AccessDenied) => false,
            Err(ClipboardError::UnsupportedPlatform) => false,
            Err(_) => true, // Other errors mean clipboard works but has no text
        }
    });

    commands.spawn(ClipboardAvailabilityTask(task));
}

/// System to handle clipboard availability test completion - following async_compute.rs pattern
fn handle_clipboard_availability_task(
    mut commands: Commands,
    mut availability_tasks: Query<(Entity, &mut ClipboardAvailabilityTask)>,
    mut clipboard_res: ResMut<ClipboardResource>,
) {
    for (entity, mut task) in &mut availability_tasks {
        if let Some(available) = block_on(future::poll_once(&mut task.0)) {
            // Task is complete, update resource and remove task component
            clipboard_res.available = available;

            if available {
                info!("Clipboard service initialized successfully with arboard");
            } else {
                error!("Failed to initialize clipboard - not available on this platform");
            }

            commands.entity(entity).despawn();
        }
    }
}

/// System to handle incoming clipboard requests - creates Task components following
/// async_compute.rs
fn handle_clipboard_requests(
    mut commands: Commands,
    mut events: EventReader<ClipboardRequest>,
    clipboard_res: Res<ClipboardResource>,
) {
    let available = clipboard_res.available;

    for event in events.read() {
        let task = match event {
            ClipboardRequest::Get { format, requester } => {
                let format = *format;
                let requester = *requester;
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = if available {
                        ArboardManager::get(format).await
                    } else {
                        Err(ClipboardError::AccessDenied)
                    };
                    ClipboardResponse::GetResult { requester, result }
                })
            },
            ClipboardRequest::Set { data, requester } => {
                let data = data.clone();
                let requester = *requester;
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = if available {
                        ArboardManager::set(data).await
                    } else {
                        Err(ClipboardError::AccessDenied)
                    };
                    ClipboardResponse::SetResult { requester, result }
                })
            },
            ClipboardRequest::Clear { requester } => {
                let requester = *requester;
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = if available {
                        ArboardManager::clear().await
                    } else {
                        Err(ClipboardError::AccessDenied)
                    };
                    ClipboardResponse::ClearResult { requester, result }
                })
            },
            ClipboardRequest::CheckFormat { format, requester } => {
                let format = *format;
                let requester = *requester;
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = if available {
                        ArboardManager::has_format(format).await
                    } else {
                        false
                    };
                    ClipboardResponse::CheckFormatResult { requester, result }
                })
            },
            ClipboardRequest::GetAvailableFormats { requester } => {
                let requester = *requester;
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = if available {
                        ArboardManager::available_formats().await
                    } else {
                        vec![]
                    };
                    ClipboardResponse::AvailableFormatsResult { requester, result }
                })
            },
        };

        // Spawn entity with Task component following async_compute.rs pattern
        commands.spawn(ClipboardTask(task));
    }
}

/// System to handle completed clipboard tasks - following async_compute.rs pattern exactly
fn handle_clipboard_tasks(
    mut commands: Commands,
    mut clipboard_tasks: Query<(Entity, &mut ClipboardTask)>,
    mut response_writer: EventWriter<ClipboardResponse>,
) {
    for (entity, mut task) in &mut clipboard_tasks {
        if let Some(response) = block_on(future::poll_once(&mut task.0)) {
            // Task is complete, send response event and remove task component
            response_writer.write(response);
            commands.entity(entity).despawn();
        }
    }
}

/// System to spawn async clipboard change detection tasks
fn detect_clipboard_changes(
    mut commands: Commands,
    time: Res<Time>,
    mut watchers: Query<(Entity, &mut ClipboardWatcher), Without<ClipboardCheckTask>>,
    existing_tasks: Query<&ClipboardCheckTask>,
    clipboard_res: Res<ClipboardResource>,
) {
    if !clipboard_res.available {
        return;
    }

    for (entity, mut watcher) in &mut watchers {
        if !watcher.active {
            continue;
        }

        // Update timer
        watcher.check_interval.tick(time.delta());

        // Only spawn new task if interval elapsed and no existing task for this watcher
        if watcher.check_interval.just_finished() && 
           !existing_tasks.iter().any(|task| task.requester == entity) {
            
            let previous_sequence = watcher.last_sequence;
            let task = AsyncComputeTaskPool::get().spawn(get_clipboard_sequence());
            
            commands.spawn(ClipboardCheckTask {
                task,
                previous_sequence,
                requester: entity,
            });
        }
    }
}

/// System to process completed clipboard check tasks and emit change events
fn process_clipboard_check_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ClipboardCheckTask)>,
    mut watchers: Query<&mut ClipboardWatcher>,
    mut change_events: EventWriter<crate::types::ClipboardChangeEvent>,
) {
    for (task_entity, mut task) in &mut tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.task)) {
            match result {
                Ok(current_sequence) => {
                    // Check if sequence changed
                    if current_sequence != task.previous_sequence {
                        // Update watcher with new sequence
                        if let Ok(mut watcher) = watchers.get_mut(task.requester) {
                            watcher.last_sequence = current_sequence;
                            
                            // Emit change event
                            change_events.write(crate::types::ClipboardChangeEvent {
                                sequence: current_sequence,
                                content: None, // Lazy loading - plugins can request content
                                timestamp: std::time::Instant::now(),
                                watcher: task.requester,
                            });
                            
                            tracing::debug!("Clipboard changed: sequence {} -> {}", 
                                task.previous_sequence, current_sequence);
                        }
                    }
                },
                Err(e) => {
                    // Handle clipboard access failure
                    tracing::warn!("Clipboard access failed: {}", e);
                    
                    // Disable watcher temporarily and set re-enable timer
                    if let Ok(mut watcher) = watchers.get_mut(task.requester) {
                        watcher.active = false;
                        commands.entity(task.requester).insert(ClipboardWatcherReenableTimer::default());
                    }
                }
            }
            
            // Clean up completed task
            commands.entity(task_entity).despawn();
        }
    }
}

/// System to re-enable failed clipboard watchers after delay
fn reenable_clipboard_watchers(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut ClipboardWatcherReenableTimer)>,
    mut watchers: Query<&mut ClipboardWatcher>,
) {
    for (entity, mut timer) in &mut timers {
        timer.timer.tick(time.delta());
        
        if timer.timer.just_finished() {
            // Re-enable the watcher
            if let Ok(mut watcher) = watchers.get_mut(entity) {
                watcher.active = true;
                tracing::info!("Re-enabled clipboard watcher after failure recovery");
            }
            
            // Remove the timer component
            commands.entity(entity).remove::<ClipboardWatcherReenableTimer>();
        }
    }
}

/// Startup system to create default clipboard watcher
fn spawn_default_clipboard_watcher(mut commands: Commands) {
    commands.spawn(ClipboardWatcher::default());
}
