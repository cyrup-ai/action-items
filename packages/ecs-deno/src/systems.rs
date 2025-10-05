//! Deno Runtime ECS Systems
//!
//! Systems for processing Deno operations, managing runtime pools, handling async tasks,
//! and integrating with Bevy's task system for optimal performance.

use bevy::{
    prelude::*,
    tasks::futures_lite::future,
};
use std::time::Instant;

use crate::{
    events::*,
    resources::*,
    components::*,
};

/// Type alias for complex timeout query to improve readability
type TimeoutQueryItem<'a> = (
    Entity,
    &'a mut crate::components::DenoOperationTimeout,
    &'a DenoOperationContext,
    Option<&'a DenoScriptExecution>,
    Option<&'a ExtensionDiscoveryOperation>,
);

/// System for managing the Deno runtime pool health and lifecycle
pub fn manage_deno_runtime_pool_system(
    pool: ResMut<DenoRuntimePool>,
    mut metrics: ResMut<DenoMetrics>,
) {
    // Get current pool health
    let health = pool.as_ref().get_health_info();
    
    // Update metrics
    metrics.update_runtime_metrics(health.available_runtimes, health.utilization_percent);
    
    // Log health information periodically (could be controlled by a timer)
    if health.utilization_percent > 90.0 {
        warn!(
            "Deno runtime pool utilization high: {:.1}% ({}/{})",
            health.utilization_percent,
            health.max_runtimes - health.available_runtimes,
            health.max_runtimes
        );
    }
}

/// System for processing script execution requests and spawning async tasks
pub fn process_script_execution_requests_system(
    mut commands: Commands,
    mut events: EventReader<DenoScriptExecutionRequested>,
    mut tracker: ResMut<DenoOperationTracker>,
    runtime_pool: Res<DenoRuntimePool>,
) {
    for request in events.read() {
        debug!(
            "Processing script execution request: {} from {}",
            request.operation_id, request.requester
        );

        // Track the operation
        let tracked_operation = TrackedOperation {
            operation_type: OperationType::ScriptExecution,
            started_at: request.requested_at,
            timeout: request.timeout,
            requester: request.requester.clone(),
            metadata: std::collections::HashMap::new(),
        };
        tracker.start_operation(request.operation_id, tracked_operation);

        // Spawn entity with components for this operation
        let mut entity_commands = commands.spawn(ScriptExecutionBundle::new(
            request.operation_id,
            request.script_content.clone(),
            request.requester.clone(),
            request.timeout,
        ));

        // Create execution request and receiver for secure runtime pool execution
        use tokio::sync::oneshot;
        let (result_sender, result_receiver) = oneshot::channel();
        
        // Create execution request for the runtime pool
        let execution_request = crate::resources::DenoExecutionRequest {
            operation_id: request.operation_id,
            script: request.script_content.clone(),
            timeout: request.timeout,
            response_sender: result_sender,
        };
        
        // Submit execution request to the secure runtime pool
        if let Err(send_error) = runtime_pool.submit_execution_request(execution_request) {
            error!("Failed to submit script execution request: {}", send_error);
            continue;
        }

        // Spawn the async task to wait for execution result
        let task = DenoScriptTask::spawn_script_execution_with_receiver(
            request.operation_id,
            request.timeout,
            result_receiver,
        );

        // Add the task component to the entity
        entity_commands.insert(task);

        info!(
            "Spawned script execution task for operation: {} (timeout: {:?})",
            request.operation_id, request.timeout
        );
    }
}

/// System for processing completed script execution tasks
pub fn process_script_execution_completions_system(
    mut commands: Commands,
    mut script_tasks: Query<(Entity, &mut DenoScriptTask)>,
    _tracker: ResMut<DenoOperationTracker>,
    _metrics: ResMut<DenoMetrics>,
) {
    for (entity, mut task) in &mut script_tasks {
        // Poll the task to see if it's completed
        if let Some(mut command_queue) = bevy::tasks::block_on(future::poll_once(&mut task.0)) {
            // Task completed, execute the queued commands
            commands.append(&mut command_queue);
            
            // Remove the entity as the task is complete
            commands.entity(entity).despawn();
        }
    }
}

/// System for processing extension discovery requests
pub fn process_extension_discovery_requests_system(
    mut commands: Commands,
    mut events: EventReader<ExtensionDiscoveryRequested>,
    mut tracker: ResMut<DenoOperationTracker>,
    discovery_manager: Res<ExtensionDiscoveryManager>,
) {
    if !discovery_manager.enabled {
        // Skip processing if discovery is disabled
        return;
    }

    for request in events.read() {
        debug!(
            "Processing extension discovery request: {} from {} ({} paths)",
            request.operation_id, request.requester, request.search_paths.len()
        );

        // Track the operation
        let tracked_operation = TrackedOperation {
            operation_type: OperationType::ExtensionDiscovery,
            started_at: request.requested_at,
            timeout: std::time::Duration::from_secs(60), // Default discovery timeout
            requester: request.requester.clone(),
            metadata: std::collections::HashMap::new(),
        };
        tracker.start_operation(request.operation_id, tracked_operation);

        // Spawn entity with components for this operation
        let mut entity_commands = commands.spawn(ExtensionDiscoveryBundle::new(
            request.operation_id,
            request.search_paths.clone(),
            request.requester.clone(),
        ));

        // Spawn the async discovery task
        let task = DenoDiscoveryTask::spawn_discovery_operation(
            request.operation_id,
            request.search_paths.clone(),
        );

        // Add the task component to the entity
        entity_commands.insert(task);

        info!(
            "Spawned extension discovery task for operation: {} ({} paths)",
            request.operation_id, request.search_paths.len()
        );
    }
}

/// System for processing completed extension discovery tasks
pub fn process_extension_discovery_completions_system(
    mut commands: Commands,
    mut discovery_tasks: Query<(Entity, &mut DenoDiscoveryTask)>,
    _tracker: ResMut<DenoOperationTracker>,
    _metrics: ResMut<DenoMetrics>,
) {
    for (entity, mut task) in &mut discovery_tasks {
        // Poll the task to see if it's completed
        if let Some(mut command_queue) = bevy::tasks::block_on(future::poll_once(&mut task.0)) {
            // Task completed, execute the queued commands
            commands.append(&mut command_queue);
            
            // Remove the entity as the task is complete
            commands.entity(entity).despawn();
        }
    }
}

/// System for handling operation timeouts
pub fn handle_operation_timeouts_system(
    mut commands: Commands,
    mut timeout_query: Query<TimeoutQueryItem>,
    mut timeout_events: EventWriter<crate::events::DenoOperationTimeout>,
    mut tracker: ResMut<DenoOperationTracker>,
) {
    for (entity, mut timeout, context, script_execution, discovery_operation) in &mut timeout_query {
        if timeout.is_expired() {
            timeout.mark_timed_out();
            
            // Get the actual operation_id from the appropriate component
            let operation_id = if let Some(script) = script_execution {
                script.operation_id
            } else if let Some(discovery) = discovery_operation {
                discovery.operation_id
            } else {
                // Fallback: generate UUID only if no operation component found
                uuid::Uuid::new_v4()
            };
            
            warn!(
                "Operation timeout for {} ({}): {:?}",
                context.requester, 
                match context.operation_type {
                    DenoOperationType::ScriptExecution => "script execution",
                    DenoOperationType::ExtensionDiscovery => "extension discovery", 
                    DenoOperationType::RuntimeManagement => "runtime management",
                    DenoOperationType::MetricsCollection => "metrics collection",
                },
                timeout.timeout_duration
            );

            // Send timeout event with actual operation_id
            timeout_events.write(crate::events::DenoOperationTimeout {
                operation_id,
                operation_type: context.operation_type.clone(),
                timeout_duration: timeout.timeout_duration,
                elapsed_time: timeout.timeout_duration, // Since it timed out
                requester: context.requester.clone(),
                timed_out_at: Instant::now(),
            });

            // Complete the operation in tracker with actual operation_id
            tracker.complete_operation(operation_id, false, Some("Timeout".to_string()));

            // Remove the entity
            commands.entity(entity).despawn();
        }
    }
}

/// System for cleaning up completed operations and managing resources
pub fn cleanup_completed_operations_system(
    tracker: ResMut<DenoOperationTracker>,
    _time: Res<Time>,
) {
    // This system runs periodically to clean up old completed operations
    // and perform maintenance tasks
    
    // The operation tracker already limits history size, but we could do additional cleanup here
    // such as removing very old completed operations or compacting metrics
    
    let stats = tracker.get_statistics();
    if stats.total_completed > 10000 {
        debug!("Operation tracker has {} completed operations, considering cleanup", stats.total_completed);
    }
}

/// System for updating and collecting Deno metrics
pub fn update_deno_metrics_system(
    mut metrics: ResMut<DenoMetrics>,
    pool: Res<DenoRuntimePool>,
    tracker: Res<DenoOperationTracker>,
    _time: Res<Time>,
) {
    // Update runtime metrics from pool
    let health = pool.as_ref().get_health_info();
    metrics.update_runtime_metrics(health.available_runtimes, health.utilization_percent);
    
    // Update operation statistics
    let operation_stats = tracker.get_statistics();
    
    // Update execution metrics (simplified - would be more sophisticated in real implementation)
    if operation_stats.total_completed > 0 {
        metrics.execution_metrics.total_executions = operation_stats.total_completed;
        metrics.execution_metrics.successful_executions = operation_stats.successful_count;
        metrics.execution_metrics.failed_executions = operation_stats.failed_count;
        metrics.execution_metrics.average_execution_time = operation_stats.average_duration;
    }
}

/// System for handling script execution completion events
pub fn handle_script_execution_completions_system(
    mut completion_events: EventReader<DenoScriptExecutionCompleted>,
    mut failure_events: EventReader<DenoScriptExecutionFailed>,
    mut tracker: ResMut<DenoOperationTracker>,
    mut metrics: ResMut<DenoMetrics>,
) {
    // Handle successful completions
    for event in completion_events.read() {
        info!(
            "Script execution completed: {} (time: {:?}, memory: {} bytes)",
            event.operation_id, event.execution_time, event.memory_used
        );

        // Update operation tracker
        tracker.complete_operation(event.operation_id, true, None);
        
        // Update metrics
        metrics.record_execution(event.execution_time, true);
    }

    // Handle failures
    for event in failure_events.read() {
        warn!(
            "Script execution failed: {} (time: {:?}, error: {:?})",
            event.operation_id, event.execution_time, event.error
        );

        // Update operation tracker
        let error_message = format!("{:?}", event.error);
        tracker.complete_operation(event.operation_id, false, Some(error_message));
        
        // Update metrics
        metrics.record_execution(event.execution_time, false);
        
        // Record error in metrics
        let error_type = match &event.error {
            DenoExecutionError::ScriptError(_) => "script_error",
            DenoExecutionError::Timeout => "timeout",
            DenoExecutionError::SecurityViolation(_) => "security_violation",
            DenoExecutionError::ResourceExhausted(_) => "resource_exhausted",
            DenoExecutionError::RuntimeError(_) => "runtime_error",
            DenoExecutionError::ModuleError(_) => "module_error",
            DenoExecutionError::PermissionDenied(_) => "permission_denied",
            DenoExecutionError::IoError(_) => "io_error",
        };
        metrics.error_metrics.record_error(error_type, ErrorCategory::Execution);
    }
}

/// System for handling extension discovery completion events
pub fn handle_extension_discovery_completions_system(
    mut completion_events: EventReader<ExtensionDiscoveryCompleted>,
    mut failure_events: EventReader<ExtensionDiscoveryFailed>,
    mut tracker: ResMut<DenoOperationTracker>,
    mut metrics: ResMut<DenoMetrics>,
    mut discovery_manager: ResMut<ExtensionDiscoveryManager>,
) {
    // Handle successful discoveries
    for event in completion_events.read() {
        info!(
            "Extension discovery completed: {} ({} extensions in {:?})",
            event.operation_id, event.extensions_found, event.discovery_time
        );

        // Update operation tracker
        tracker.complete_operation(event.operation_id, true, None);
        
        // Update metrics
        metrics.record_discovery(event.discovery_time, true, event.extensions_found);
        
        // Update discovery manager stats
        discovery_manager.stats.total_discoveries += 1;
        discovery_manager.stats.successful_discoveries += 1;
        discovery_manager.stats.total_extensions_found += event.extensions_found;
    }

    // Handle failures
    for event in failure_events.read() {
        warn!(
            "Extension discovery failed: {} (time: {:?}, error: {:?})",
            event.operation_id, event.discovery_time, event.error
        );

        // Update operation tracker
        let error_message = format!("{:?}", event.error);
        tracker.complete_operation(event.operation_id, false, Some(error_message));
        
        // Update metrics
        metrics.record_discovery(event.discovery_time, false, 0);
        
        // Record error in metrics
        let error_type = match &event.error {
            ExtensionDiscoveryError::PathError(_) => "path_error",
            ExtensionDiscoveryError::ManifestError(_) => "manifest_error",
            ExtensionDiscoveryError::InvalidExtension(_) => "invalid_extension",
            ExtensionDiscoveryError::PermissionDenied(_) => "permission_denied",
            ExtensionDiscoveryError::IoError(_) => "io_error",
            ExtensionDiscoveryError::Timeout => "timeout",
            ExtensionDiscoveryError::InternalError(_) => "internal_error",
        };
        metrics.error_metrics.record_error(error_type, ErrorCategory::Discovery);
        
        // Update discovery manager stats
        discovery_manager.stats.total_discoveries += 1;
        discovery_manager.stats.failed_discoveries += 1;
    }
}

/// System for generating metrics reports when requested
pub fn handle_metrics_report_requests_system(
    mut request_events: EventReader<DenoMetricsReportRequested>,
    mut report_events: EventWriter<DenoMetricsReportGenerated>,
    metrics: Res<DenoMetrics>,
) {
    for request in request_events.read() {
        info!("Generating Deno metrics report for: {}", request.requester);
        
        let report = metrics.generate_report();
        
        report_events.write(DenoMetricsReportGenerated {
            report,
            requester: request.requester.clone(),
            generated_at: Instant::now(),
        });
    }
}