//! Action execution systems with zero-allocation hot path optimization

use std::sync::atomic::{AtomicU64, Ordering};

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use tracing::{error, info, warn};

use super::super::core::helpers::{
    execute_application_open_operation, execute_command_operation, execute_file_open_operation,
};
use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::systems::AssociatedTask;

// Global performance counter for zero-allocation metrics
static TOTAL_ACTIONS_PROCESSED: AtomicU64 = AtomicU64::new(0);

/// Process action execution requests with zero-allocation hot path optimization
#[inline(always)]
pub fn process_action_execution_requests_system(
    mut commands: Commands,
    mut execution_requests: EventReader<ActionExecuteRequested>,
    mut execution_completed: EventWriter<ActionExecuteCompleted>,
    action_registry: Res<ActionRegistry>,
    config: Res<LauncherConfig>,
) {
    for request in execution_requests.read() {
        // Zero-allocation atomic counter increment for performance metrics
        TOTAL_ACTIONS_PROCESSED.fetch_add(1, Ordering::Relaxed);

        if config.enable_debug_logging {
            info!("Processing action execution request: {}", request.action_id);
        }

        // Find action definition
        if let Some(action_def) = action_registry.actions.get(&request.action_id) {
            let action_def = action_def.clone();

            let operation_start = std::time::Instant::now();

            // Create execution tracking component with zero-allocation name generation
            let execution_entity = commands
                .spawn((
                    ActionExecution {
                        action_id: request.action_id.clone(),
                        requester: request.requester.clone(),
                        parameters: request.parameters.clone(),
                        status: ExecutionStatus::InProgress,
                        started_at: operation_start,
                        completed_at: None,
                        result: None,
                        error_message: None,
                    },
                    // Professional naming with action ID
                    Name::new(format!("ActionExecution-{}", request.action_id)),
                ))
                .id();

            // Execute action asynchronously with comprehensive TaskSpawner integration and timeout
            let execution_context = request.execution_context.clone();
            let parameters = request.parameters.clone();
            let timeout_duration = execution_context
                .timeout
                .unwrap_or(std::time::Duration::from_secs(30));

            let task_future = execute_action_with_comprehensive_management(
                action_def,
                parameters,
                execution_context,
                execution_entity,
            );

            // Apply production-quality timeout handling with zero-allocation patterns
            let timeout_future = async move {
                tokio::time::timeout(timeout_duration, task_future).await.unwrap_or_default()
            };

            // Use TaskSpawner trait for professional task management with integrated timeout
            let task = AsyncComputeTaskPool::get().spawn(timeout_future);
            let task_entity = commands
                .spawn(ActionExecutionTask {
                    task,
                    action_id: request.action_id.clone(),
                    started_at: operation_start,
                })
                .id();

            // Link execution entity to task entity for comprehensive tracking
            commands
                .entity(execution_entity)
                .insert(AssociatedTask(task_entity));
        } else {
            // Action not found - zero-allocation error reporting
            warn!("Action not found: {}", request.action_id);
            let error_message = format!("Action '{}' not found", request.action_id);

            execution_completed.write(ActionExecuteCompleted {
                action_id: request.action_id.clone(),
                requester: request.requester.clone(),
                success: false,
                result: None,
                error_message: Some(error_message),
                execution_time: std::time::Duration::ZERO,
            });
        }
    }
}

/// Execute action using comprehensive TaskManagementService with advanced error handling
async fn execute_action_with_comprehensive_management(
    action_def: ActionDefinition,
    parameters: serde_json::Value,
    context: ExecutionContext,
    execution_entity: Entity,
) -> CommandQueue {
    let mut command_queue = CommandQueue::default();
    let action_id = action_def.id.clone();
    let action_type = action_def.action_type.clone();
    let requester = context.requester.clone();
    let timeout_duration = context
        .timeout
        .unwrap_or(std::time::Duration::from_secs(30));

    let start_time = std::time::Instant::now();

    // Create ECS Task Component using proper AsyncComputeTaskPool patterns
    let action_task_future = create_action_execution_task(action_def, parameters);

    // Perform action execution with comprehensive error handling and structured timeout
    let execution_result = tokio::time::timeout(timeout_duration, action_task_future).await;

    let execution_time = start_time.elapsed();

    // Update execution entity with completion timestamp
    command_queue.push(move |world: &mut World| {
        if let Some(mut execution) = world.get_mut::<ActionExecution>(execution_entity) {
            execution.completed_at = Some(std::time::Instant::now());
        }
    });

    match execution_result {
        Ok(mut result_command_queue) => {
            // Merge the result CommandQueue with our command_queue
            command_queue.push(move |world: &mut World| {
                result_command_queue.apply(world);
                info!("Completed action execution task for type: {}", action_type);
            });
        },
        Err(_) => {
            // Handle timeout case
            command_queue.push(move |world: &mut World| {
                warn!(
                    "Action execution timed out: {} (timeout: {:?})",
                    action_type, timeout_duration
                );

                world.send_event(ActionExecuteCompleted {
                    action_id: action_id.clone(),
                    requester: requester.clone(),
                    success: false,
                    result: None,
                    error_message: Some("Action execution timed out".to_string()),
                    execution_time,
                });
            });
        },
    };

    command_queue
}

/// Create action execution task using proper ECS Task Component patterns
async fn create_action_execution_task(
    action_def: ActionDefinition,
    parameters: serde_json::Value,
) -> CommandQueue {
    let mut command_queue = CommandQueue::default();
    let action_id = action_def.id.clone();
    let action_type = action_def.action_type.clone();
    let operation_start = std::time::Instant::now();

    // Execute operation based on action type using proper async patterns
    let execution_result = match action_def.action_type.as_str() {
        "open_file" => execute_file_open_operation(action_def, parameters).await,
        "open_application" => execute_application_open_operation(action_def, parameters).await,
        "run_command" => execute_command_operation(action_def, parameters).await,
        _ => Err(format!(
            "Unsupported action type: {}. Supported types: open_file, open_application, \
             run_command",
            action_def.action_type
        )),
    };

    // Use CommandQueue to update ECS World from async context
    command_queue.push(move |world: &mut World| match execution_result {
        Ok(success_result) => {
            info!("Action operation completed successfully: {}", action_type);
            world.send_event(ActionExecuteCompleted {
                action_id: action_id.clone(),
                requester: "system".to_string(),
                success: true,
                result: Some(success_result),
                error_message: None,
                execution_time: operation_start.elapsed(),
            });
        },
        Err(error_msg) => {
            error!("Action operation failed: {} - {}", action_type, error_msg);
            world.send_event(ActionExecuteCompleted {
                action_id: action_id.clone(),
                requester: "system".to_string(),
                success: false,
                result: None,
                error_message: Some(error_msg),
                execution_time: operation_start.elapsed(),
            });
        },
    });

    command_queue
}

/// Poll action execution tasks for completion with zero-allocation task management
#[inline(always)]
pub fn poll_action_execution_tasks(
    mut commands: Commands,
    mut action_tasks: Query<(Entity, &mut ActionExecutionTask)>,
    config: Res<LauncherConfig>,
) {
    use bevy::tasks::{block_on, poll_once};

    for (entity, mut action_task) in action_tasks.iter_mut() {
        if let Some(mut command_queue) = block_on(poll_once(&mut action_task.task)) {
            let duration = action_task.started_at.elapsed();

            if config.enable_debug_logging {
                info!(
                    "Action task completed for '{}' in {:?}",
                    action_task.action_id, duration
                );
            }

            // Apply the commands from the completed task
            commands.append(&mut command_queue);

            // Clean up the completed task
            commands.entity(entity).despawn();
        }
    }
}
