//! Task management plugin following ARCHITECTURE.md service pattern

use bevy::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::systems::*;

/// Plugin for task management following ARCHITECTURE.md pattern
pub struct TaskManagementPlugin;

impl Plugin for TaskManagementPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize resources
            .init_resource::<TaskStatistics>()
            .init_resource::<TaskManagementConfig>()
            // Add events following request/response pattern
            .add_event::<HotkeyPreferencesLoadRequested>()
            .add_event::<HotkeyPreferencesLoadCompleted>()
            .add_event::<HotkeyPreferencesPersistRequested>()
            .add_event::<HotkeyPreferencesPersistCompleted>()
            .add_event::<TaskSpawnedEvent>()
            .add_event::<TaskStarted>()
            .add_event::<TaskFailed>()
            .add_event::<TaskExpired>()
            // Add core task polling systems
            .add_systems(
                Update,
                (
                    handle_task_spawned_events,
                    poll_hotkey_preferences_load_tasks,
                    poll_hotkey_preferences_persist_tasks,
                    poll_storage_operation_tasks,
                    poll_clipboard_operation_tasks,
                    poll_search_operation_tasks,
                    poll_raycast_operation_tasks,
                    poll_tls_operation_tasks,
                    poll_cache_cleanup_tasks,
                    poll_generic_tasks,
                    log_task_statistics,
                ),
            )
            // Add result cleanup systems
            .add_systems(
                Update,
                (
                    cleanup_task_results_hotkey_preferences,
                    cleanup_task_results_pathbuf,
                    cleanup_task_results_string,
                    cleanup_task_results_vec_string,
                    cleanup_task_results_unit,
                    cleanup_task_results_u64,
                ),
            )
            // Add timer processing systems
            .add_systems(
                Update,
                (
                    process_cleanup_timers::<HotkeyPreferencesResult>,
                    process_cleanup_timers::<std::path::PathBuf>,
                    process_cleanup_timers::<String>,
                    process_cleanup_timers::<Vec<String>>,
                    process_cleanup_timers::<()>,
                    process_cleanup_timers::<u64>,
                ),
            )
            // Add health and maintenance systems
            .add_systems(
                Update,
                (
                    monitor_task_execution_health,
                    maintain_task_queues,
                ),
            );
    }
}
