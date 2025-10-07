//! Systems for Service Bridge Integration
//!
//! Proper Bevy ECS systems following patterns from ecs-clipboard, ecs-notifications, and
//! ecs-service-bridge

// Import real ECS service events
use action_items_ecs_clipboard::{ClipboardRequest, ClipboardResponse};
use action_items_ecs_permissions::{PermissionRequest, PermissionChanged};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{block_on, futures_lite::future};
use ecs_service_bridge::events::{PluginMessageEvent, MessagePriority};
use ecs_service_bridge::types::TimeStamp;
use tracing::{debug, warn, info};
use uuid::Uuid;

use super::resources::{PluginMessageCorrelation, PluginOperationInfo, ServiceBridgeState};
use super::entity_mapping::PluginEntityMap;
use super::events::{NotificationSent, NotificationDeliveryStatus, PluginResponseEvent, TaskStatus};
use super::components::{PluginMessageTask, OperationTimeoutTimer};
use super::payload_parsing::{
    parse_permission_type, validate_clipboard_data
};

/// SystemParam grouping service event writers to reduce function parameter count
#[derive(SystemParam)]
pub struct ServiceEventWriters<'w> {
    clipboard_writer: EventWriter<'w, ClipboardRequest>,
    notification_writer: EventWriter<'w, ecs_notifications::components::platform::NotificationRequest>,
    permission_writer: EventWriter<'w, PermissionRequest>,
    plugin_message_writer: EventWriter<'w, PluginMessageEvent>,
}

/// System to route plugin messages to appropriate ECS services
/// Following the pattern from ecs-service-bridge's process_plugin_messages_system
pub fn plugin_message_router_system(
    mut plugin_messages: EventReader<PluginMessageEvent>,
    mut correlation: ResMut<PluginMessageCorrelation>,
    mut state: ResMut<ServiceBridgeState>,
    mut entity_map: ResMut<PluginEntityMap>,
    mut commands: Commands,
    mut writers: ServiceEventWriters,
) {
    for message in plugin_messages.read() {
        state.messages_processed += 1;

        debug!(
            "Processing plugin message: {} -> {} (type: {})",
            message.from, message.to, message.message_type
        );

        // Generate operation ID for correlation
        let operation_id = Uuid::new_v4();
        let requester_entity = match entity_map.convert_plugin_id_to_entity(&message.plugin_id, &mut commands) {
            Ok(entity) => entity,
            Err(err) => {
                warn!("Failed to convert plugin ID '{}' to entity: {}. Skipping message.", message.plugin_id, err);
                continue;
            }
        };

        // Store operation info for response correlation
        correlation.add_operation(operation_id, PluginOperationInfo {
            operation_id,
            plugin_id: message.from.clone(),
            message_type: message.message_type.clone(),
            requester_entity,
            original_request_id: message.request_id.clone(),
        });

        // Route to appropriate ECS service based on message type
        match message.message_type.as_str() {
            "clipboard_read" => {
                let clipboard_request = ClipboardRequest::Get {
                    format: action_items_ecs_clipboard::ClipboardFormat::Text,
                    requester: requester_entity,
                };
                writers.clipboard_writer.write(clipboard_request);
                state.messages_routed += 1;
                debug!("Routed clipboard read request to ecs-clipboard service");
            },
            "clipboard_write" => {
                // Extract clipboard data from JSON payload
                let data_str = match message.payload
                    .as_object()
                    .and_then(|obj| obj.get("data"))
                    .and_then(|v| v.as_str()) {
                    Some(data) => data,
                    None => {
                        warn!("Missing data field in clipboard_write message from plugin {}", message.from);
                        
                        // Send error response back to plugin
                        let error_response = PluginMessageEvent {
                            from: "system".to_string(),
                            to: message.from.clone(),
                            message_type: "clipboard_write_error".to_string(),
                            payload: serde_json::json!({
                                "success": false,
                                "error": "Missing required 'data' field in clipboard_write message"
                            }),
                            timestamp: TimeStamp::now(),
                            plugin_id: message.from.clone(),
                            priority: MessagePriority::default(),
                            request_id: message.request_id.clone(),
                            correlation_id: Some(operation_id.to_string()),
                        };
                        writers.plugin_message_writer.write(error_response);
                        continue;
                    }
                };

                let validated_data = match validate_clipboard_data(data_str) {
                    Ok(data) => data,
                    Err(e) => {
                        warn!("Invalid clipboard data from plugin {}: {}", message.from, e);
                        
                        // Send error response back to plugin
                        let error_response = PluginMessageEvent {
                            from: "system".to_string(),
                            to: message.from.clone(),
                            message_type: "clipboard_write_error".to_string(),
                            payload: serde_json::json!({
                                "success": false,
                                "error": format!("Invalid clipboard data: {}", e)
                            }),
                            timestamp: TimeStamp::now(),
                            plugin_id: message.from.clone(),
                            priority: MessagePriority::default(),
                            request_id: message.request_id.clone(),
                            correlation_id: Some(operation_id.to_string()),
                        };
                        writers.plugin_message_writer.write(error_response);
                        continue;
                    }
                };

                let clipboard_request = ClipboardRequest::Set {
                    data: action_items_ecs_clipboard::ClipboardData::Text(validated_data),
                    requester: requester_entity,
                };
                writers.clipboard_writer.write(clipboard_request);
                state.messages_routed += 1;
                debug!("Routed clipboard write request to ecs-clipboard service");
            },
            "clipboard_clear" => {
                let clipboard_request = ClipboardRequest::Clear {
                    requester: requester_entity,
                };
                writers.clipboard_writer.write(clipboard_request);
                state.messages_routed += 1;
                debug!("Routed clipboard clear request to ecs-clipboard service");
            },
            "notification_show" => {
                // Parse notification data from JSON payload
                let title = message.payload
                    .as_object()
                    .and_then(|obj| obj.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Notification");
                
                let body = message.payload
                    .as_object()
                    .and_then(|obj| obj.get("body"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message content");

                let notification_request =
                    ecs_notifications::components::platform::NotificationRequest {
                        notification_id: format!("notif_{}", uuid::Uuid::new_v4()),
                        content: ecs_notifications::NotificationContent::new(
                            title,
                            ecs_notifications::RichText::plain(body),
                        ),
                        options: ecs_notifications::DeliveryOptions::default(),
                        correlation_id: operation_id.to_string(),
                    };
                writers.notification_writer.write(notification_request);
                state.messages_routed += 1;
                debug!("Routed notification request to ecs-notifications service with title '{}' and body '{}'", title, body);
            },
            "permission_check" | "permission_request" => {
                // Extract permission type from JSON payload
                let permission_type_str = match message.payload
                    .as_object()
                    .and_then(|obj| obj.get("permission_type"))
                    .and_then(|v| v.as_str()) {
                    Some(perm_type) => perm_type,
                    None => {
                        warn!("Missing permission_type field in permission request from plugin {}", message.from);
                        
                        // Send error response back to plugin
                        let error_response = PluginMessageEvent {
                            from: "system".to_string(),
                            to: message.from.clone(),
                            message_type: "permission_error".to_string(),
                            payload: serde_json::json!({
                                "success": false,
                                "error": "Missing required 'permission_type' field in permission request"
                            }),
                            timestamp: TimeStamp::now(),
                            plugin_id: message.from.clone(),
                            priority: MessagePriority::default(),
                            request_id: message.request_id.clone(),
                            correlation_id: Some(operation_id.to_string()),
                        };
                        writers.plugin_message_writer.write(error_response);
                        continue;
                    }
                };

                let permission_type = match parse_permission_type(permission_type_str) {
                    Ok(perm_type) => perm_type,
                    Err(e) => {
                        warn!("Invalid permission type from plugin {}: {}", message.from, e);
                        
                        // Send error response back to plugin
                        let error_response = PluginMessageEvent {
                            from: "system".to_string(),
                            to: message.from.clone(),
                            message_type: "permission_error".to_string(),
                            payload: serde_json::json!({
                                "success": false,
                                "error": format!("Invalid permission type: {}", e)
                            }),
                            timestamp: TimeStamp::now(),
                            plugin_id: message.from.clone(),
                            priority: MessagePriority::default(),
                            request_id: message.request_id.clone(),
                            correlation_id: Some(operation_id.to_string()),
                        };
                        writers.plugin_message_writer.write(error_response);
                        continue;
                    }
                };

                let permission_request = PermissionRequest {
                    typ: permission_type,
                };
                writers.permission_writer.write(permission_request);
                state.messages_routed += 1;
                debug!("Routed permission request to ecs-permissions service");
            },
            _ => {
                warn!("Unsupported message type '{}' from plugin {}", message.message_type, message.from);
                
                // Send error response back to plugin
                let error_response = PluginMessageEvent {
                    from: "system".to_string(),
                    to: message.from.clone(),
                    message_type: "message_type_error".to_string(),
                    payload: serde_json::json!({
                        "success": false,
                        "error": format!("Unsupported message type: {}", message.message_type),
                        "supported_types": ["clipboard_read", "clipboard_write", "clipboard_clear", "notification_show", "permission_check", "permission_request"]
                    }),
                    timestamp: TimeStamp::now(),
                    plugin_id: message.from.clone(),
                    priority: MessagePriority::default(),
                    request_id: message.request_id.clone(),
                    correlation_id: Some(operation_id.to_string()),
                };
                writers.plugin_message_writer.write(error_response);
                continue;
            },
        }

        state.active_operations += 1;
    }
}

/// System to handle ECS service responses and correlate them back to plugins
/// Processes responses from clipboard, permissions, and notification services
pub fn ecs_service_integration_system(
    mut correlation: ResMut<PluginMessageCorrelation>,
    mut state: ResMut<ServiceBridgeState>,
    mut plugin_message_writer: EventWriter<PluginMessageEvent>,
    mut clipboard_responses: EventReader<ClipboardResponse>,
    mut permission_changes: EventReader<PermissionChanged>,
    mut notification_sent: EventReader<NotificationSent>,
    entity_map: Res<PluginEntityMap>,
) {
    // Handle clipboard service responses
    for clipboard_response in clipboard_responses.read() {
        let requester_entity = match clipboard_response {
            ClipboardResponse::GetResult { requester, result } => {
                debug!("Received clipboard get result for entity {:?}", requester);
                handle_clipboard_get_response(*requester, result, &mut correlation, &mut plugin_message_writer, &entity_map, &mut state);
                *requester
            },
            ClipboardResponse::SetResult { requester, result } => {
                debug!("Received clipboard set result for entity {:?}", requester);
                handle_clipboard_set_response(*requester, result, &mut correlation, &mut plugin_message_writer, &entity_map, &mut state);
                *requester
            },
            ClipboardResponse::ClearResult { requester, result } => {
                debug!("Received clipboard clear result for entity {:?}", requester);
                handle_clipboard_clear_response(*requester, result, &mut correlation, &mut plugin_message_writer, &entity_map, &mut state);
                *requester
            },
            ClipboardResponse::CheckFormatResult { requester, result } => {
                debug!("Received clipboard check format result for entity {:?}", requester);
                handle_clipboard_check_format_response(*requester, *result, &mut correlation, &mut plugin_message_writer, &entity_map, &mut state);
                *requester
            },
            ClipboardResponse::AvailableFormatsResult { requester, result } => {
                debug!("Received clipboard available formats result for entity {:?}", requester);
                handle_clipboard_formats_response(*requester, result, &mut correlation, &mut plugin_message_writer, &entity_map, &mut state);
                *requester
            },
        };

        state.responses_processed += 1;
        debug!("Processed clipboard response for entity {:?}", requester_entity);
    }

    // Handle permission service responses
    for permission_change in permission_changes.read() {
        debug!("Received permission change: {:?} -> {:?}", permission_change.typ, permission_change.status);

        // Find correlations for this permission type and forward to requesting plugins
        let correlations_to_resolve = correlation.find_correlations_by_message_type(&format!("permission_check_{:?}", permission_change.typ));

        for operation_info in correlations_to_resolve {
            if let Some(plugin_id) = entity_map.get_plugin_id(operation_info.requester_entity) {
                // Create permission response message
                let response_message = PluginMessageEvent {
                    from: "system".to_string(),
                    to: plugin_id.clone(),
                    message_type: "permission_response".to_string(),
                    payload: serde_json::json!({
                        "permission_type": format!("{:?}", permission_change.typ),
                        "status": format!("{:?}", permission_change.status),
                        "success": true
                    }),
                    timestamp: TimeStamp::now(),
                    plugin_id: plugin_id.clone(),
                    priority: MessagePriority::default(),
                    request_id: operation_info.original_request_id.clone().or_else(|| Some(operation_info.operation_id.to_string())),
                    correlation_id: Some(operation_info.operation_id.to_string()),
                };

                plugin_message_writer.write(response_message);
                correlation.remove_operation(&operation_info.operation_id);
                info!("Forwarded permission change to plugin: {}", plugin_id);
            }
        }

        state.responses_processed += 1;
    }

    // Handle notification service responses
    for notification in notification_sent.read() {
        debug!("Received notification sent: {} -> {:?}", notification.notification_id, notification.delivery_status);

        // Find correlations for notification requests and forward to requesting plugins
        let correlations_to_resolve = correlation.find_correlations_by_message_type("notification_show");

        for operation_info in correlations_to_resolve.iter().filter(|op| op.plugin_id == notification.plugin_id) {
            // Create notification response message
            let response_message = PluginMessageEvent {
                from: "system".to_string(),
                to: notification.plugin_id.clone(),
                message_type: "notification_response".to_string(),
                payload: serde_json::json!({
                    "notification_id": notification.notification_id,
                    "delivery_status": format!("{:?}", notification.delivery_status),
                    "success": notification.delivery_status == NotificationDeliveryStatus::Delivered,
                    "error": notification.error_message
                }),
                timestamp: TimeStamp::now(),
                plugin_id: notification.plugin_id.clone(),
                priority: MessagePriority::default(),
                request_id: operation_info.original_request_id.clone().or_else(|| Some(operation_info.operation_id.to_string())),
                correlation_id: Some(operation_info.operation_id.to_string()),
            };

            plugin_message_writer.write(response_message);
            correlation.remove_operation(&operation_info.operation_id);
            info!("Forwarded notification status to plugin: {}", notification.plugin_id);
        }

        state.responses_processed += 1;
    }

    debug!("ECS service integration system processed {} total responses", state.responses_processed);
}

/// Handle clipboard get operation response
fn handle_clipboard_get_response(
    requester: Entity,
    result: &Result<action_items_ecs_clipboard::ClipboardData, action_items_ecs_clipboard::ClipboardError>,
    correlation: &mut PluginMessageCorrelation,
    plugin_writer: &mut EventWriter<PluginMessageEvent>,
    entity_map: &PluginEntityMap,
    state: &mut ServiceBridgeState,
) {
    if let Some(plugin_id) = entity_map.get_plugin_id(requester) {
        let (success, payload) = match result {
            Ok(clipboard_data) => {
                let data_json = match clipboard_data {
                    action_items_ecs_clipboard::ClipboardData::Text(text) => serde_json::json!({"type": "text", "content": text}),
                    action_items_ecs_clipboard::ClipboardData::Html { html, alt_text: _ } => serde_json::json!({"type": "html", "content": html}),
                    _ => serde_json::json!({"type": "unknown", "content": "binary data"}),
                };
                (true, data_json)
            },
            Err(err) => (false, serde_json::json!({"error": format!("{:?}", err)})),
        };

        // Get operation info for proper correlation
        let operation_info = correlation.find_operation_by_entity(requester);
        let (correlation_id, request_id) = if let Some(info) = operation_info {
            (
                Some(info.operation_id.to_string()),
                info.original_request_id.clone().or_else(|| Some(info.operation_id.to_string())),
            )
        } else {
            (None, None)
        };

        let response_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "clipboard_read_response".to_string(),
            payload: serde_json::json!({"success": success, "data": payload}),
            timestamp: TimeStamp::now(),
            plugin_id: plugin_id.clone(),
            priority: MessagePriority::default(),
            request_id,
            correlation_id,
        };

        plugin_writer.write(response_message);

        // Clean up correlation for this requester
        correlation.cleanup_operations_for_entity(requester);
        state.active_operations = state.active_operations.saturating_sub(1);
    }
}

/// Handle clipboard set operation response
fn handle_clipboard_set_response(
    requester: Entity,
    result: &Result<(), action_items_ecs_clipboard::ClipboardError>,
    correlation: &mut PluginMessageCorrelation,
    plugin_writer: &mut EventWriter<PluginMessageEvent>,
    entity_map: &PluginEntityMap,
    state: &mut ServiceBridgeState,
) {
    if let Some(plugin_id) = entity_map.get_plugin_id(requester) {
        let (success, error) = match result {
            Ok(()) => (true, None),
            Err(err) => (false, Some(format!("{:?}", err))),
        };

        // Get operation info for proper correlation
        let operation_info = correlation.find_operation_by_entity(requester);
        let (correlation_id, request_id) = if let Some(info) = operation_info {
            (
                Some(info.operation_id.to_string()),
                info.original_request_id.clone().or_else(|| Some(info.operation_id.to_string())),
            )
        } else {
            (None, None)
        };

        let response_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "clipboard_write_response".to_string(),
            payload: serde_json::json!({"success": success, "error": error}),
            timestamp: TimeStamp::now(),
            plugin_id: plugin_id.clone(),
            priority: MessagePriority::default(),
            request_id,
            correlation_id,
        };

        plugin_writer.write(response_message);

        // Clean up correlation for this requester
        correlation.cleanup_operations_for_entity(requester);
        state.active_operations = state.active_operations.saturating_sub(1);
    }
}

/// Handle clipboard clear operation response
fn handle_clipboard_clear_response(
    requester: Entity,
    result: &Result<(), action_items_ecs_clipboard::ClipboardError>,
    correlation: &mut PluginMessageCorrelation,
    plugin_writer: &mut EventWriter<PluginMessageEvent>,
    entity_map: &PluginEntityMap,
    state: &mut ServiceBridgeState,
) {
    if let Some(plugin_id) = entity_map.get_plugin_id(requester) {
        let (success, error) = match result {
            Ok(()) => (true, None),
            Err(err) => (false, Some(format!("{:?}", err))),
        };

        let response_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "clipboard_clear_response".to_string(),
            payload: serde_json::json!({"success": success, "error": error}),
            timestamp: TimeStamp::now(),
            plugin_id: plugin_id.clone(),
            priority: MessagePriority::default(),
            request_id: None,
            correlation_id: None,
        };

        plugin_writer.write(response_message);

        // Clean up correlation for this requester
        correlation.cleanup_operations_for_entity(requester);
        state.active_operations = state.active_operations.saturating_sub(1);
    }
}

/// Handle clipboard check format operation response
fn handle_clipboard_check_format_response(
    requester: Entity,
    result: bool,
    correlation: &mut PluginMessageCorrelation,
    plugin_writer: &mut EventWriter<PluginMessageEvent>,
    entity_map: &PluginEntityMap,
    state: &mut ServiceBridgeState,
) {
    if let Some(plugin_id) = entity_map.get_plugin_id(requester) {
        let response_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "clipboard_check_format_response".to_string(),
            payload: serde_json::json!({"success": true, "has_format": result}),
            timestamp: TimeStamp::now(),
            plugin_id: plugin_id.clone(),
            priority: MessagePriority::default(),
            request_id: None,
            correlation_id: None,
        };

        plugin_writer.write(response_message);

        // Clean up correlation for this requester
        correlation.cleanup_operations_for_entity(requester);
        state.active_operations = state.active_operations.saturating_sub(1);
    }
}

/// Handle clipboard available formats operation response
fn handle_clipboard_formats_response(
    requester: Entity,
    result: &[action_items_ecs_clipboard::ClipboardFormat],
    correlation: &mut PluginMessageCorrelation,
    plugin_writer: &mut EventWriter<PluginMessageEvent>,
    entity_map: &PluginEntityMap,
    state: &mut ServiceBridgeState,
) {
    if let Some(plugin_id) = entity_map.get_plugin_id(requester) {
        let formats: Vec<String> = result.iter().map(|f| format!("{:?}", f)).collect();

        let response_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "clipboard_formats_response".to_string(),
            payload: serde_json::json!({"success": true, "formats": formats}),
            timestamp: TimeStamp::now(),
            plugin_id: plugin_id.clone(),
            priority: MessagePriority::default(),
            request_id: None,
            correlation_id: None,
        };

        plugin_writer.write(response_message);

        // Clean up correlation for this requester
        correlation.cleanup_operations_for_entity(requester);
        state.active_operations = state.active_operations.saturating_sub(1);
    }
}

/// System to handle async task completion following async_compute.rs pattern
/// Processes PluginMessageTask components with proper state management, timeouts, and cleanup
pub fn async_task_handler_system(
    mut commands: Commands,
    mut plugin_message_writer: EventWriter<PluginMessageEvent>,
    mut task_query: Query<(Entity, &mut PluginMessageTask)>,
    mut correlation: ResMut<PluginMessageCorrelation>,
    mut state: ResMut<ServiceBridgeState>,
) {
    let mut completed_tasks = Vec::new();
    let mut timed_out_tasks = Vec::new();
    let mut failed_tasks: Vec<(Entity, Uuid, String)> = Vec::new();

    // Process all active plugin message tasks
    for (task_entity, mut plugin_task) in task_query.iter_mut() {
        // Check for timeout first
        if plugin_task.is_timed_out() && plugin_task.status.is_active() {
            warn!("Plugin task timed out: {} (plugin: {})", plugin_task.operation_id, plugin_task.plugin_id);
            plugin_task.timeout();
            timed_out_tasks.push((task_entity, plugin_task.operation_id, plugin_task.plugin_id.clone()));
            continue;
        }

        // Only process pending or running tasks
        if !plugin_task.status.is_active() {
            continue;
        }

        // Mark task as running if it was pending
        if plugin_task.status == TaskStatus::Pending {
            plugin_task.start();
            debug!("Started plugin task: {} (plugin: {})", plugin_task.operation_id, plugin_task.plugin_id);
        }

        // Poll the task using Bevy's recommended async pattern with proper failure detection
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            block_on(future::poll_once(&mut plugin_task.task))
        })) {
            Ok(Some(result)) => {
                // Task completed successfully
                debug!("Plugin task completed: {} (plugin: {})", plugin_task.operation_id, plugin_task.plugin_id);
                plugin_task.complete();

                // Send the result as a plugin message response
                let response_message = PluginMessageEvent {
                    from: "system".to_string(),
                    to: plugin_task.plugin_id.clone(),
                    message_type: "async_task_response".to_string(),
                    payload: serde_json::json!({
                        "operation_id": plugin_task.operation_id,
                        "success": true,
                        "message": result
                    }),
                    timestamp: TimeStamp::now(),
                    plugin_id: plugin_task.plugin_id.clone(),
                    priority: MessagePriority::default(),
                    request_id: Some(plugin_task.operation_id.to_string()),
                    correlation_id: Some(plugin_task.operation_id.to_string()),
                };

                plugin_message_writer.write(response_message);
                completed_tasks.push((task_entity, plugin_task.operation_id));
            },
            Ok(None) => {
                // Task is still running, continue processing in next frame
                debug!("Plugin task still running: {} (plugin: {})", plugin_task.operation_id, plugin_task.plugin_id);
            },
            Err(panic_payload) => {
                // Task failed with panic - handle failure
                let error_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Task panicked with unknown payload".to_string()
                };

                warn!("Plugin task failed with panic: {} (plugin: {}) - Error: {}",
                      plugin_task.operation_id, plugin_task.plugin_id, error_msg);
                plugin_task.fail();

                failed_tasks.push((task_entity, plugin_task.operation_id, error_msg));
            }
        }
    }

    // Capture counts before moving the vectors
    let completed_count = completed_tasks.len();
    let timed_out_count = timed_out_tasks.len();
    let failed_count = failed_tasks.len();

    // Clean up completed tasks
    for (task_entity, operation_id) in completed_tasks {
        commands.entity(task_entity).despawn();
        correlation.remove_operation(&operation_id);
        state.active_operations = state.active_operations.saturating_sub(1);
        debug!("Cleaned up completed task entity {:?}", task_entity);
    }

    // Handle timed out tasks
    for (task_entity, operation_id, plugin_id) in timed_out_tasks {
        // Send timeout response to plugin
        let timeout_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "async_task_timeout".to_string(),
            payload: serde_json::json!({
                "operation_id": operation_id,
                "success": false,
                "error": "Task timed out after maximum duration"
            }),
            timestamp: TimeStamp::now(),
            plugin_id,
            priority: MessagePriority::default(),
            request_id: Some(operation_id.to_string()),
            correlation_id: Some(operation_id.to_string()),
        };

        plugin_message_writer.write(timeout_message);

        // Clean up timed out task
        commands.entity(task_entity).despawn();
        correlation.remove_operation(&operation_id);
        state.active_operations = state.active_operations.saturating_sub(1);
        debug!("Cleaned up timed out task entity {:?}", task_entity);
    }

    // Handle failed tasks (if any)
    for (task_entity, operation_id, plugin_id) in failed_tasks {
        // Send failure response to plugin
        let failure_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "async_task_failure".to_string(),
            payload: serde_json::json!({
                "operation_id": operation_id,
                "success": false,
                "error": "Task failed during execution"
            }),
            timestamp: TimeStamp::now(),
            plugin_id,
            priority: MessagePriority::default(),
            request_id: Some(operation_id.to_string()),
            correlation_id: Some(operation_id.to_string()),
        };

        plugin_message_writer.write(failure_message);

        // Clean up failed task
        commands.entity(task_entity).despawn();
        correlation.remove_operation(&operation_id);
        state.active_operations = state.active_operations.saturating_sub(1);
        debug!("Cleaned up failed task entity {:?}", task_entity);
    }

    // Update metrics
    if completed_count > 0 || timed_out_count > 0 || failed_count > 0 {
        debug!(
            "Async task handler processed {} completed, {} timed out, {} failed tasks",
            completed_count,
            timed_out_count,
            failed_count
        );
    }
}

/// System to correlate ECS service responses back to plugins
/// Processes PluginResponseEvent, matches correlation IDs, and handles timeouts
pub fn response_correlation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut correlation: ResMut<PluginMessageCorrelation>,
    mut state: ResMut<ServiceBridgeState>,
    mut plugin_message_writer: EventWriter<PluginMessageEvent>,
    mut plugin_responses: EventReader<PluginResponseEvent>,
    mut timeout_query: Query<(Entity, &mut OperationTimeoutTimer)>,
) {
    let mut completed_operations = Vec::new();
    let mut timed_out_operations = Vec::new();
    let mut orphaned_responses = Vec::new();

    // Process incoming plugin response events
    for response_event in plugin_responses.read() {
        debug!(
            "Processing plugin response: {} (plugin: {}, type: {})",
            response_event.operation_id, response_event.plugin_id, response_event.response_type
        );

        // Try to find the correlation for this response
        match correlation.get_operation(&response_event.operation_id) {
            Some(operation_info) => {
                // Found matching operation, process the response
                if operation_info.plugin_id == response_event.plugin_id {
                    // Create response message to forward back to the plugin
                    let response_message = PluginMessageEvent {
                        from: "system".to_string(),
                        to: response_event.plugin_id.clone(),
                        message_type: format!("{}_response", response_event.response_type),
                        payload: serde_json::json!({
                            "operation_id": response_event.operation_id,
                            "success": response_event.success,
                            "data": response_event.payload,
                            "error": response_event.error,
                            "metadata": response_event.metadata,
                            "timestamp": response_event.timestamp
                        }),
                        timestamp: TimeStamp::now(),
                        plugin_id: response_event.plugin_id.clone(),
                        priority: MessagePriority::default(),
                        request_id: None,
                        correlation_id: None,
                    };

                    plugin_message_writer.write(response_message);
                    completed_operations.push(response_event.operation_id);

                    info!(
                        "Correlated response for operation {} to plugin {}",
                        response_event.operation_id, response_event.plugin_id
                    );
                } else {
                    warn!(
                        "Operation {} belongs to plugin {} but response came from {}",
                        response_event.operation_id, operation_info.plugin_id, response_event.plugin_id
                    );
                    orphaned_responses.push(response_event.clone());
                }
            },
            None => {
                // No matching operation found - this is an orphaned response
                warn!(
                    "Received orphaned response for operation {} from plugin {}",
                    response_event.operation_id, response_event.plugin_id
                );
                orphaned_responses.push(response_event.clone());
            }
        }
    }

    // Update timeout timers and check for timed-out operations
    for (timer_entity, mut timeout_timer) in timeout_query.iter_mut() {
        // Update the timer
        timeout_timer.timer.tick(time.delta());

        // Check if operation has timed out
        if timeout_timer.is_timed_out() && timeout_timer.status.is_active() {
            warn!(
                "Operation {} timed out for plugin {}",
                timeout_timer.operation_id, timeout_timer.plugin_id
            );

            timeout_timer.timeout();
            timed_out_operations.push((
                timer_entity,
                timeout_timer.operation_id,
                timeout_timer.plugin_id.clone(),
            ));
        }
    }

    // Capture counts before moving the vectors
    let completed_count = completed_operations.len();
    let timed_out_count = timed_out_operations.len();
    let orphaned_count = orphaned_responses.len();

    // Clean up completed operations
    for operation_id in completed_operations {
        // Remove from correlation tracking
        correlation.remove_operation(&operation_id);
        state.responses_correlated += 1;

        // Find and clean up associated timeout timer
        for (timer_entity, timeout_timer) in timeout_query.iter() {
            if timeout_timer.operation_id == operation_id {
                commands.entity(timer_entity).despawn();
                debug!("Cleaned up timeout timer for completed operation {}", operation_id);
                break;
            }
        }
    }

    // Handle timed out operations
    for (timer_entity, operation_id, plugin_id) in timed_out_operations {
        // Send timeout notification to plugin
        let timeout_message = PluginMessageEvent {
            from: "system".to_string(),
            to: plugin_id.clone(),
            message_type: "operation_timeout".to_string(),
            payload: serde_json::json!({
                "operation_id": operation_id,
                "success": false,
                "error": "Operation timed out after 30 seconds",
                "timeout_duration_secs": 30
            }),
            timestamp: TimeStamp::now(),
            plugin_id: plugin_id.clone(),
            priority: MessagePriority::default(),
            request_id: None,
            correlation_id: None,
        };

        plugin_message_writer.write(timeout_message);

        // Clean up timed out operation
        correlation.remove_operation(&operation_id);
        commands.entity(timer_entity).despawn();
        state.active_operations = state.active_operations.saturating_sub(1);

        info!("Cleaned up timed out operation {} for plugin {}", operation_id, plugin_id);
    }

    // Handle orphaned responses
    for orphaned_response in orphaned_responses {
        // Log orphaned response for debugging
        debug!(
            "Orphaned response: {} from plugin {} (no matching operation)",
            orphaned_response.operation_id, orphaned_response.plugin_id
        );

        // Optionally send notification to the plugin about the orphaned response
        let orphan_message = PluginMessageEvent {
            from: "system".to_string(),
            to: orphaned_response.plugin_id.clone(),
            message_type: "orphaned_response".to_string(),
            payload: serde_json::json!({
                "operation_id": orphaned_response.operation_id,
                "success": false,
                "error": "No matching operation found for this response",
                "original_response": orphaned_response.payload
            }),
            timestamp: TimeStamp::now(),
            plugin_id: orphaned_response.plugin_id.clone(),
            priority: MessagePriority::default(),
            request_id: None,
            correlation_id: None,
        };

        plugin_message_writer.write(orphan_message);
    }

    // Update metrics and logging
    let active_operations = correlation.active_operation_count();
    if active_operations != state.active_operations as usize {
        state.active_operations = active_operations as u32;
    }

    if completed_count > 0 || timed_out_count > 0 || orphaned_count > 0 {
        debug!(
            "Response correlation system processed {} responses, {} timeouts, {} orphans. Active operations: {}",
            completed_count,
            timed_out_count,
            orphaned_count,
            state.active_operations
        );
    }
}
