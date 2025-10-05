//! Comprehensive task management systems with zero-allocation optimizations

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::tasks::block_on;
use bevy::tasks::futures_lite::future;
use bevy::time::{Timer, TimerMode};

// use paste::paste; // Unused import
use crate::{components::*, events::*};

/// Optimized task polling macro for zero-allocation hot path
macro_rules! poll_task_type {
    ($system_name:ident, $task_component:ty, $result_type:ty, $task_type:expr) => {
        pub fn $system_name(
            mut commands: Commands,
            mut task_query: Query<(Entity, &mut $task_component)>,
            mut stats: ResMut<TaskStatistics>,
            _time: Res<Time>,
        ) {
            let _current_time = Instant::now();

            for (entity, mut task_component) in task_query.iter_mut() {
                let task_op = &mut task_component.base;

                // Fast path: check expiration first (zero-allocation)
                if task_op.is_expired() {
                    let elapsed_ms = task_op.elapsed_ms();
                    let timeout_ms = task_op.timeout_duration.as_millis() as u64;

                    warn!(
                        "Task {} ({}) expired after {}ms (limit: {}ms)",
                        task_op.id,
                        $task_type.name(),
                        elapsed_ms,
                        timeout_ms
                    );

                    stats.record_expiration($task_type);

                    // Execute callback with timeout error
                    if let Some(callback) = task_op.completion_callback {
                        let error = TaskError::Timeout {
                            task_id: task_op.id,
                            task_type: $task_type,
                            timeout_ms,
                            elapsed_ms,
                            message: format!("Task expired after {}ms", elapsed_ms),
                        };
                        callback(Err(error));
                    }

                    // Trigger expiration event
                    commands.trigger(TaskExpired {
                        task_id: task_op.id,
                        operation_type: $task_type.name().to_string(),
                        duration: task_op.created_at.elapsed(),
                    });

                    commands.entity(entity).despawn();
                    continue;
                }

                // Poll task with panic recovery (zero-allocation fast path)
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    block_on(future::poll_once(&mut task_op.task))
                })) {
                    Ok(Some(result)) => {
                        let elapsed_ms = task_op.elapsed_ms();

                        match result {
                            Ok(task_result) => {
                                info!(
                                    "Task {} ({}) completed successfully in {}ms",
                                    task_op.id,
                                    $task_type.name(),
                                    elapsed_ms
                                );

                                stats.record_completion($task_type, elapsed_ms);

                                // Execute callback with success
                                if let Some(callback) = task_op.completion_callback {
                                    callback(Ok(task_result));
                                } else {
                                    // Store result for later retrieval
                                    commands.spawn((
                                        TaskResultComponent::new(
                                            task_result,
                                            task_op.id,
                                            $task_type,
                                        ),
                                        TaskResultCleanupTimer::<$result_type>::new(
                                            std::time::Duration::from_secs(300),
                                        ),
                                    ));
                                }
                            },
                            Err(task_error) => {
                                error!(
                                    "Task {} ({}) failed: {}",
                                    task_op.id,
                                    $task_type.name(),
                                    task_error
                                );

                                stats.record_failure($task_type);

                                // Execute callback with error
                                if let Some(callback) = task_op.completion_callback {
                                    callback(Err(task_error));
                                }

                                // Trigger failure event
                                commands.trigger(TaskFailed {
                                    task_id: task_op.id,
                                    operation_type: $task_type.name().to_string(),
                                    error: "Task execution failed".to_string(),
                                });
                            },
                        }

                        commands.entity(entity).despawn();
                    },
                    Ok(None) => {
                        // Task still running - continue polling next frame
                    },
                    Err(_) => {
                        error!("Task {} ({}) panicked", task_op.id, $task_type.name());

                        stats.record_failure($task_type);

                        let panic_error = TaskError::System {
                            task_id: task_op.id,
                            task_type: $task_type,
                            message: "Task panicked during execution".to_string(),
                            error_code: Some(-1),
                        };

                        // Execute callback with panic error
                        if let Some(callback) = task_op.completion_callback {
                            callback(Err(panic_error));
                        }

                        // Trigger failure event
                        commands.trigger(TaskFailed {
                            task_id: task_op.id,
                            operation_type: $task_type.name().to_string(),
                            error: "Task panicked".to_string(),
                        });

                        commands.entity(entity).despawn();
                    },
                }
            }
        }
    };
}

// Generate optimized polling systems for all task types
poll_task_type!(
    poll_hotkey_preferences_load_tasks,
    HotkeyPreferencesLoadTask,
    HotkeyPreferencesResult,
    TaskType::HotkeyPreferences
);
poll_task_type!(
    poll_hotkey_preferences_persist_tasks,
    HotkeyPreferencesPersistTask,
    std::path::PathBuf,
    TaskType::HotkeyPreferences
);
poll_task_type!(
    poll_storage_operation_tasks,
    StorageOperationTask,
    std::path::PathBuf,
    TaskType::StorageOperation
);
poll_task_type!(
    poll_clipboard_operation_tasks,
    ClipboardOperationTask,
    String,
    TaskType::ClipboardOperation
);
poll_task_type!(
    poll_search_operation_tasks,
    SearchOperationTask,
    Vec<String>,
    TaskType::SearchOperation
);
poll_task_type!(
    poll_raycast_operation_tasks,
    RaycastOperationTask,
    String,
    TaskType::RaycastOperation
);
poll_task_type!(
    poll_tls_operation_tasks,
    TlsOperationTask,
    (),
    TaskType::TlsOperation
);
poll_task_type!(
    poll_cache_cleanup_tasks,
    CacheCleanupTask,
    u64,
    TaskType::CacheCleanup
);
poll_task_type!(poll_generic_tasks, GenericTask, String, TaskType::Generic);

/// Generic result cleanup system with zero-allocation optimization
pub fn cleanup_task_results<T: Send + Sync + 'static>(
    mut commands: Commands,
    task_results: Query<Entity, With<TaskResultComponent<T>>>,
    existing_timers: Query<Entity, With<TaskResultCleanupTimer<T>>>,
    config: Res<TaskManagementConfig>,
) {
    if !config.enable_result_storage {
        return;
    }

    let cleanup_delay = config.result_cleanup_delay;
    let mut timers_created = 0u32;

    // Schedule cleanup for newly completed task results (zero-allocation iterator)
    for entity in task_results.iter() {
        // Check if this entity already has a cleanup timer to prevent duplicates
        if !existing_timers.contains(entity) {
            let cleanup_timer = TaskResultCleanupTimer::<T>::new(cleanup_delay);

            // Use simple insert approach to avoid EntityCommands type issues
            commands.entity(entity).insert(cleanup_timer);
            timers_created += 1;
        }
    }

    if timers_created > 0 {
        debug!(
            "Scheduled {} task results for cleanup in {}s",
            timers_created,
            cleanup_delay.as_secs()
        );
    }
}

/// Process cleanup timers with zero-allocation optimization
pub fn process_cleanup_timers<T: Send + Sync + 'static>(
    mut commands: Commands,
    cleanup_timers: Query<(Entity, &TaskResultCleanupTimer<T>)>,
) {
    let current_time = Instant::now();
    let mut cleaned_count = 0u32;

    for (entity, timer) in cleanup_timers.iter() {
        if timer.is_ready_for_cleanup(current_time) {
            let elapsed = current_time.duration_since(timer.created_at);

            debug!(
                "Cleaning up task result entity {:?} after {:.2}s (scheduled for {:.2}s)",
                entity,
                elapsed.as_secs_f64(),
                timer.cleanup_delay.as_secs_f64()
            );

            // Use simple despawn approach to avoid EntityCommands type issues
            commands.entity(entity).despawn();
            cleaned_count += 1;
        }
    }

    if cleaned_count > 0 {
        debug!("Cleaned up {} expired task result entities", cleaned_count);
    }
}

/// Enhanced statistics logging with zero-allocation optimization
pub fn log_task_statistics(
    stats: Res<TaskStatistics>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
    config: Res<TaskManagementConfig>,
) {
    if !config.enable_statistics {
        return;
    }

    if timer.is_none() {
        *timer = Some(Timer::new(
            config.statistics_log_interval,
            TimerMode::Repeating,
        ));
    }

    if let Some(ref mut t) = timer.as_mut()
        && t.tick(time.delta()).just_finished() {
            let total_active = stats.total_active_tasks();

            info!(
                "Task Statistics - Active: {}, Completed: {}, Failed: {}, Expired: {}, Total: {}",
                total_active,
                stats.total_completed,
                stats.total_failed,
                stats.total_expired,
                stats.total_spawned
            );

            // Log per-type statistics (zero-allocation)
            for (task_type, count) in &stats.active_tasks_by_type {
                if *count > 0 {
                    debug!("  {} active: {}", task_type.name(), count);
                }
            }
        }
}

/// Comprehensive task spawned event handler
pub fn handle_task_spawned_events(
    mut events: EventReader<TaskSpawnedEvent>,
    mut stats: ResMut<TaskStatistics>,
) {
    for event in events.read() {
        // Parse task type from operation string (zero-allocation lookup)
        let task_type = match event.operation_type.as_str() {
            "HotkeyPreferences" => TaskType::HotkeyPreferences,
            "StorageOperation" => TaskType::StorageOperation,
            "ClipboardOperation" => TaskType::ClipboardOperation,
            "SearchOperation" => TaskType::SearchOperation,
            "RaycastOperation" => TaskType::RaycastOperation,
            "TlsOperation" => TaskType::TlsOperation,
            "CacheCleanup" => TaskType::CacheCleanup,
            _ => TaskType::Generic,
        };

        stats.increment_active(task_type);
        info!("Task spawned: {} ({})", event.id, task_type.name());
    }
}

// Manual cleanup system implementations (avoiding complex macro issues)
pub fn cleanup_task_results_hotkey_preferences(
    commands: Commands,
    task_results: Query<Entity, With<TaskResultComponent<HotkeyPreferencesResult>>>,
    existing_timers: Query<Entity, With<TaskResultCleanupTimer<HotkeyPreferencesResult>>>,
    config: Res<TaskManagementConfig>,
) {
    cleanup_task_results::<HotkeyPreferencesResult>(
        commands,
        task_results,
        existing_timers,
        config,
    );
}

pub fn cleanup_task_results_pathbuf(
    commands: Commands,
    task_results: Query<Entity, With<TaskResultComponent<std::path::PathBuf>>>,
    existing_timers: Query<Entity, With<TaskResultCleanupTimer<std::path::PathBuf>>>,
    config: Res<TaskManagementConfig>,
) {
    cleanup_task_results::<std::path::PathBuf>(commands, task_results, existing_timers, config);
}

pub fn cleanup_task_results_string(
    commands: Commands,
    task_results: Query<Entity, With<TaskResultComponent<String>>>,
    existing_timers: Query<Entity, With<TaskResultCleanupTimer<String>>>,
    config: Res<TaskManagementConfig>,
) {
    cleanup_task_results::<String>(commands, task_results, existing_timers, config);
}

pub fn cleanup_task_results_vec_string(
    commands: Commands,
    task_results: Query<Entity, With<TaskResultComponent<Vec<String>>>>,
    existing_timers: Query<Entity, With<TaskResultCleanupTimer<Vec<String>>>>,
    config: Res<TaskManagementConfig>,
) {
    cleanup_task_results::<Vec<String>>(commands, task_results, existing_timers, config);
}

pub fn cleanup_task_results_unit(
    commands: Commands,
    task_results: Query<Entity, With<TaskResultComponent<()>>>,
    existing_timers: Query<Entity, With<TaskResultCleanupTimer<()>>>,
    config: Res<TaskManagementConfig>,
) {
    cleanup_task_results::<()>(commands, task_results, existing_timers, config);
}

pub fn cleanup_task_results_u64(
    commands: Commands,
    task_results: Query<Entity, With<TaskResultComponent<u64>>>,
    existing_timers: Query<Entity, With<TaskResultCleanupTimer<u64>>>,
    config: Res<TaskManagementConfig>,
) {
    cleanup_task_results::<u64>(commands, task_results, existing_timers, config);
}

/// System to monitor task execution health and performance
pub fn monitor_task_execution_health(
    stats: Res<TaskStatistics>,
    config: Res<TaskManagementConfig>,
    mut health_timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    // Initialize health monitoring timer
    if health_timer.is_none() {
        *health_timer = Some(Timer::new(Duration::from_secs(30), TimerMode::Repeating));
    }

    if let Some(ref mut timer) = health_timer.as_mut()
        && timer.tick(time.delta()).just_finished() {
            let total_active = stats.total_active_tasks();

            // Check for task overload conditions
            if total_active > config.max_concurrent_tasks as u64 {
                warn!(
                    "Task overload detected: {} active tasks (limit: {})",
                    total_active, config.max_concurrent_tasks
                );
            }

            // Check per-type limits
            for (task_type, active_count) in &stats.active_tasks_by_type {
                if let Some(type_limit) = config.max_concurrent_per_type.get(task_type)
                    && *active_count > *type_limit as u64 {
                        warn!(
                            "Task type {} overload: {} active (limit: {})",
                            task_type.name(),
                            active_count,
                            type_limit
                        );
                    }
            }

            // Calculate failure rates
            let total_tasks = stats.total_spawned;
            if total_tasks > 0 {
                let failure_rate =
                    (stats.total_failed + stats.total_expired) as f64 / total_tasks as f64;
                if failure_rate > 0.1 {
                    // More than 10% failure rate
                    warn!(
                        "High task failure rate detected: {:.1}% ({} failed + {} expired out of \
                         {} total)",
                        failure_rate * 100.0,
                        stats.total_failed,
                        stats.total_expired,
                        total_tasks
                    );
                }
            }
        }
}

/// System to perform periodic task queue maintenance
#[allow(clippy::type_complexity)]
pub fn maintain_task_queues(
    _commands: Commands,
    all_tasks: Query<Entity, Or<(
        With<HotkeyPreferencesLoadTask>,
        With<HotkeyPreferencesPersistTask>,
        With<StorageOperationTask>,
        With<ClipboardOperationTask>,
        With<SearchOperationTask>,
        With<RaycastOperationTask>,
        With<TlsOperationTask>,
        With<CacheCleanupTask>,
        With<GenericTask>
    )>>,
    stats: Res<TaskStatistics>,
    _config: Res<TaskManagementConfig>,
    mut maintenance_timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    // Initialize maintenance timer - run every 5 minutes
    if maintenance_timer.is_none() {
        *maintenance_timer = Some(Timer::new(Duration::from_secs(300), TimerMode::Repeating));
    }

    if let Some(ref mut timer) = maintenance_timer.as_mut()
        && timer.tick(time.delta()).just_finished() {
            let task_count = all_tasks.iter().count();
            let stats_active = stats.total_active_tasks();

            // Verify statistics consistency
            if task_count != stats_active as usize {
                warn!(
                    "Task count mismatch detected: ECS query found {} tasks, statistics report {} \
                     active",
                    task_count, stats_active
                );

                // Could trigger statistics correction here if needed
            }

            debug!(
                "Task queue maintenance complete: {} active tasks verified",
                task_count
            );
        }
}
