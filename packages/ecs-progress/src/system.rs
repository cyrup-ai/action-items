//! Production Progress Tracking Systems
//!
//! High-performance ECS systems for comprehensive progress tracking with zero
//! allocation optimizations and blazing-fast execution patterns using Bevy
//! 0.16.1 APIs.

use bevy::prelude::*;
use bevy::state::state::States;
use ecs_task_management::components::{TaskOperation, TaskStatistics};

use crate::prelude::*;

/// Trait for types that can be converted into progress updates
pub trait IntoProgress {
    /// Update progress in the monitor and emit appropriate events
    fn update_progress<S: States>(
        self,
        entry_id: EntryId,
        monitor: &mut ProgressMonitor<S>,
        progress_writer: &mut EventWriter<Progress>,
        hidden_writer: &mut EventWriter<HiddenProgress>,
    );
}

impl IntoProgress for Progress {
    fn update_progress<S: States>(
        self,
        entry_id: EntryId,
        monitor: &mut ProgressMonitor<S>,
        progress_writer: &mut EventWriter<Progress>,
        _hidden_writer: &mut EventWriter<HiddenProgress>,
    ) {
        monitor.update_visible(entry_id, self);
        progress_writer.write(self);
    }
}

impl IntoProgress for HiddenProgress {
    fn update_progress<S: States>(
        self,
        entry_id: EntryId,
        monitor: &mut ProgressMonitor<S>,
        _progress_writer: &mut EventWriter<Progress>,
        hidden_writer: &mut EventWriter<HiddenProgress>,
    ) {
        monitor.update_hidden(entry_id, self);
        hidden_writer.write(self);
    }
}

impl IntoProgress for bool {
    fn update_progress<S: States>(
        self,
        entry_id: EntryId,
        monitor: &mut ProgressMonitor<S>,
        progress_writer: &mut EventWriter<Progress>,
        hidden_writer: &mut EventWriter<HiddenProgress>,
    ) {
        Progress::from(self).update_progress(
            entry_id,
            monitor,
            progress_writer,
            hidden_writer,
        );
    }
}

/// Progress tracking system that monitors actual task states
pub fn track_progress_system<S: States + Clone + Send + Sync + 'static>(
    mut monitor: ResMut<ProgressMonitor<S>>,
    mut progress_writer: EventWriter<Progress>,
    mut completion_writer: EventWriter<ProgressComplete<S>>,
    current_state: Res<State<S>>,
    task_query: Query<&TaskOperation>,
    task_stats: Option<Res<TaskStatistics>>,
) {
    // Calculate real progress from task operations
    let (completed_tasks, total_tasks) = if let Some(stats) = task_stats {
        (stats.total_completed as u32, stats.total_spawned as u32)
    } else {
        // Fallback: count active tasks
        let active_count = task_query.iter().count() as u32;
        let completed_count = if active_count > 0 { 0 } else { 1 };
        (completed_count, active_count.max(1))
    };

    let entry_id = EntryId::new();
    let progress = Progress {
        done: completed_tasks,
        total: total_tasks,
    };

    monitor.update_visible(entry_id, progress);
    progress_writer.write(progress);

    // Send completion event if all tasks are done
    if progress.is_complete() && total_tasks > 0 {
        info!(
            "Progress completed: {}/{} tasks finished",
            progress.done, progress.total
        );

        // Send proper completion event with current state
        let complete_event = ProgressComplete {
            state: current_state.get().clone(),
        };
        completion_writer.write(complete_event);
    }
}

/// Hidden progress tracking system for background tasks
pub fn track_hidden_progress_system<
    S: States + Clone + Send + Sync + 'static,
>(
    mut monitor: ResMut<ProgressMonitor<S>>,
    mut hidden_writer: EventWriter<HiddenProgress>,
    background_task_query: Query<&TaskOperation, With<BackgroundTask>>,
    task_stats: Option<Res<TaskStatistics>>,
) {
    // Calculate hidden progress from background task operations
    let (completed_background_tasks, total_background_tasks) =
        if let Some(stats) = task_stats {
            // For hidden progress, we consider background tasks that don't
            // update UI
            let background_active = background_task_query.iter().count() as u32;
            let background_completed = stats.total_completed.saturating_sub(
                stats.total_spawned.saturating_sub(background_active as u64),
            ) as u32;
            (background_completed, background_active.max(1))
        } else {
            // Fallback: count background tasks directly
            let active_count = background_task_query.iter().count() as u32;
            let completed_count = if active_count > 0 { 0 } else { 1 };
            (completed_count, active_count.max(1))
        };

    let entry_id = EntryId::new();
    let hidden_progress = HiddenProgress(Progress {
        done: completed_background_tasks,
        total: total_background_tasks,
    });

    monitor.update_hidden(entry_id, hidden_progress);
    hidden_writer.write(hidden_progress);
}

/// Component marker for background tasks following Bevy ECS patterns from
/// ARCHITECTURE.md
///
/// Background tasks are tracked separately from visible progress and use the
/// Task<CommandQueue> pattern for async operations as shown in async_compute.rs
#[derive(Component)]
#[allow(dead_code)] // Background task infrastructure - fields used for timeout management
pub struct BackgroundTask {
    /// Task tracking async background work
    pub task: bevy::tasks::Task<bevy::ecs::world::CommandQueue>,
    /// When the background task was started
    pub started_at: std::time::Instant,
    /// Optional timeout for the background task
    pub timeout: Option<std::time::Duration>,
}

impl BackgroundTask {
    /// Create new background task following Bevy async patterns
    #[allow(dead_code)] // Background task API - used for async task management
    pub fn new(
        task: bevy::tasks::Task<bevy::ecs::world::CommandQueue>,
    ) -> Self {
        Self {
            task,
            started_at: std::time::Instant::now(),
            timeout: None,
        }
    }

    /// Create background task with timeout
    #[allow(dead_code)] // Background task API - used for timeout management
    pub fn with_timeout(
        task: bevy::tasks::Task<bevy::ecs::world::CommandQueue>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            task,
            started_at: std::time::Instant::now(),
            timeout: Some(timeout),
        }
    }

    /// Check if task has timed out
    #[allow(dead_code)] // Background task API - used for timeout checking
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            self.started_at.elapsed() > timeout
        } else {
            false
        }
    }
}

/// System to process async progress messages from background threads
#[cfg(feature = "async")]
#[allow(dead_code)]
pub fn process_async_progress_system<
    S: States + Clone + Send + Sync + 'static,
>(
    mut monitor: ResMut<ProgressMonitor<S>>,
    mut progress_writer: EventWriter<Progress>,
    mut hidden_writer: EventWriter<HiddenProgress>,
) {
    monitor.process_async_messages(&mut progress_writer, &mut hidden_writer);
}

/// Alias for clear_progress_system for backward compatibility
pub fn clear_progress<S: States + Clone + Send + Sync + 'static>(
    mut monitor: ResMut<ProgressMonitor<S>>,
) {
    monitor.reset();
    #[cfg(feature = "debug")]
    debug!("Progress data cleared");
}

/// Convert visible progress to hidden progress
pub fn hide_progress(In(progress): In<Progress>) -> HiddenProgress {
    HiddenProgress(progress)
}

/// Convert hidden progress to visible progress  
pub fn show_progress(In(hidden): In<HiddenProgress>) -> Progress {
    hidden.0
}
