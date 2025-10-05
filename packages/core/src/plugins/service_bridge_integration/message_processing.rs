use std::collections::HashMap;

use bevy::prelude::*;
use ecs_service_bridge::components::PluginStatus;
use ecs_service_bridge::events::{
    BroadcastMessageEvent, LifecycleEventType, PluginLifecycleEvent, PluginMessageEvent,
};
use ecs_service_bridge::messaging::MessageInfrastructure;
use ecs_service_bridge::types::ServiceError;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

// Import real ECS service events
use ecs_clipboard::{ClipboardRequest, ClipboardResponse};
use ecs_notifications::NotificationRequest;
use ecs_permissions::PermissionRequest;

use super::ServiceBridgeIntegration;
use super::components::ServiceBridgeRegistration;
use super::message_translator::{
    translate_ecs_response_to_message, translate_message_to_ecs_events, EcsEventResponse,
    EcsEventType,
};

// Use OperationId from message_translator to avoid duplication
use super::message_translator::OperationId;

/// Tracks pending operations for correlation between requests and responses
#[derive(Resource, Default)]
pub struct PendingOperations {
    operations: HashMap<Uuid, (String, String)>, // operation_id -> (plugin_id, message_type)
}

impl PendingOperations {
    pub fn add_operation(&mut self, operation_id: Uuid, plugin_id: String, message_type: String) {
        self.operations
            .insert(operation_id, (plugin_id, message_type));
    }

    pub fn get_operation(&self, operation_id: &Uuid) -> Option<&(String, String)> {
        self.operations.get(operation_id)
    }

    pub fn remove_operation(&mut self, operation_id: &Uuid) -> Option<(String, String)> {
        self.operations.remove(operation_id)
    }
}

/// System to process service bridge messages using ECS events
pub fn process_service_bridge_messages(
    mut plugin_messages: EventReader<PluginMessageEvent>,
    mut broadcast_messages: EventReader<BroadcastMessageEvent>,
    mut lifecycle_events: EventReader<PluginLifecycleEvent>,
    mut registrations: Query<&mut ServiceBridgeRegistration>,
    mut pending_operations: ResMut<PendingOperations>,

    // ECS service event writers
    mut clipboard_writer: EventWriter<ClipboardRequest>,
    mut notification_writer: EventWriter<NotificationRequest>,
    mut http_writer: EventWriter<HttpRequestSubmitted>,
    mut search_writer: EventWriter<SearchRequested>,
    mut permission_writer: EventWriter<PermissionRequest>,

    _message_infrastructure: Res<MessageInfrastructure>,
) {
    // Process plugin-specific messages by routing to ECS services
    for message_event in plugin_messages.read() {
        if let Some(mut registration) = registrations
            .iter_mut()
            .find(|r| r.plugin_id == message_event.to)
        {
            match route_message_to_ecs_services(
                message_event,
                &mut registration,
                &mut pending_operations,
                &mut clipboard_writer,
                &mut notification_writer,
                &mut http_writer,
                &mut search_writer,
                &mut permission_writer,
            ) {
                Ok(_) => {
                    trace!(
                        "Routed message for plugin '{}' to ECS services",
                        registration.plugin_id
                    );
                },
                Err(e) => {
                    error!(
                        "Failed to route message for plugin '{}' to ECS services: {}",
                        registration.plugin_id, e
                    );
                },
            }
        }
    }

    // Process broadcast messages
    for broadcast_event in broadcast_messages.read() {
        match process_broadcast_message_event(broadcast_event) {
            Ok(_) => {
                trace!("Processed broadcast message");
            },
            Err(e) => {
                error!("Failed to process broadcast message: {}", e);
            },
        }
    }

    // Process lifecycle events to update plugin status
    for lifecycle_event in lifecycle_events.read() {
        if let Some(mut registration) = registrations
            .iter_mut()
            .find(|r| r.plugin_id == lifecycle_event.plugin_id)
        {
            match process_lifecycle_event(lifecycle_event, &mut registration) {
                Ok(_) => {
                    debug!("Updated plugin '{}' status", registration.plugin_id);
                },
                Err(e) => {
                    error!(
                        "Failed to process lifecycle event for plugin '{}': {}",
                        registration.plugin_id, e
                    );
                },
            }
        }
    }
}

/// Route plugin message to appropriate ECS services
fn route_message_to_ecs_services(
    message_event: &PluginMessageEvent,
    registration: &mut ServiceBridgeRegistration,
    pending_operations: &mut PendingOperations,
    clipboard_writer: &mut EventWriter<ClipboardRequest>,
    notification_writer: &mut EventWriter<NotificationRequest>,
    http_writer: &mut EventWriter<HttpRequestSubmitted>,
    search_writer: &mut EventWriter<SearchRequested>,
    permission_writer: &mut EventWriter<PermissionRequest>,
) -> Result<(), ServiceError> {
    info!(
        "Routing message type '{}' from plugin '{}' to ECS services",
        message_event.message_type, registration.plugin_id
    );

    // Update plugin activity
    registration.status = PluginStatus::Active;

    // Handle lifecycle messages directly
    match message_event.message_type.as_str() {
        "start" => {
            registration.status = PluginStatus::Active;
            info!("Starting plugin '{}'", registration.plugin_id);
            return Ok(());
        },
        "stop" => {
            registration.status = PluginStatus::Inactive;
            info!("Stopping plugin '{}'", registration.plugin_id);
            return Ok(());
        },
        "status" => {
            debug!("Status request for plugin '{}'", registration.plugin_id);
            return Ok(());
        },
        _ => {},
    }

    // Generate operation ID for correlation
    let operation_id = OperationId::new();

    // Track the operation for response correlation
    pending_operations.add_operation(
        operation_id.0,
        message_event.from.clone(),
        message_event.message_type.clone(),
    );

    // Translate message to ECS events and route to appropriate services
    match translate_message_to_ecs_events(message_event, operation_id) {
        Ok(ecs_events) => {
            for ecs_event in ecs_events {
                match ecs_event {
                    EcsEventType::ClipboardRequest(req) => {
                        debug!("Routing clipboard request to ECS clipboard service");
                        clipboard_writer.send(req);
                    },
                    EcsEventType::NotificationRequest(req) => {
                        debug!("Routing notification request to ECS notification service");
                        notification_writer.send(req);
                    },
                    EcsEventType::HttpRequest(req) => {
                        debug!("Routing HTTP request to ECS fetch service");
                        http_writer.send(req);
                    },
                    EcsEventType::SearchRequest(req) => {
                        debug!("Routing search request to ECS search aggregator service");
                        search_writer.send(req);
                    },
                    EcsEventType::PermissionRequest(req) => {
                        debug!("Routing permission request to ECS permissions service");
                        permission_writer.send(req);
                    },
                }
            }
            Ok(())
        },
        Err(e) => {
            error!("Failed to translate message to ECS events: {}", e);
            return Err(ServiceError::Internal {
                reason: format!("Failed to translate message to ECS events: {}", e),
            });
        },
    }
}

/// Process a broadcast message event
fn process_broadcast_message_event(
    broadcast_event: &BroadcastMessageEvent,
) -> Result<(), ServiceError> {
    info!(
        "Received broadcast message type '{}' from '{}'",
        broadcast_event.message_type, broadcast_event.from
    );

    // Handle different broadcast message types
    match broadcast_event.message_type.as_str() {
        "system_announcement" => {
            info!("System announcement: {}", broadcast_event.payload);
        },
        "plugin_update" => {
            debug!("Plugin update broadcast: {}", broadcast_event.payload);
        },
        "shutdown" => {
            warn!("System shutdown broadcast received");
        },
        _ => {
            trace!(
                "Unknown broadcast message type: {}",
                broadcast_event.message_type
            );
        },
    }

    Ok(())
}

/// Process a lifecycle event
fn process_lifecycle_event(
    lifecycle_event: &PluginLifecycleEvent,
    registration: &mut ServiceBridgeRegistration,
) -> Result<(), ServiceError> {
    match &lifecycle_event.event_type {
        LifecycleEventType::Registered => {
            registration.status = PluginStatus::Active;
            info!(
                "Plugin '{}' registered with service bridge",
                lifecycle_event.plugin_id
            );
        },
        LifecycleEventType::Started => {
            registration.status = PluginStatus::Active;
            info!("Plugin '{}' started", lifecycle_event.plugin_id);
        },
        LifecycleEventType::Stopped => {
            registration.status = PluginStatus::Inactive;
            info!("Plugin '{}' stopped", lifecycle_event.plugin_id);
        },
        LifecycleEventType::Unregistered => {
            registration.status = PluginStatus::Inactive;
            info!(
                "Plugin '{}' unregistered from service bridge",
                lifecycle_event.plugin_id
            );
        },
        LifecycleEventType::StatusChanged(new_status) => {
            info!(
                "Plugin '{}' status changed to: {}",
                lifecycle_event.plugin_id, new_status
            );
        },
        LifecycleEventType::Error(error) => {
            registration.status = PluginStatus::Error(error.clone());
            error!("Plugin '{}' error: {}", lifecycle_event.plugin_id, error);
        },
    }

    Ok(())
}

/// System to handle ECS service responses and route them back to plugins
pub fn handle_ecs_service_responses(
    mut clipboard_responses: EventReader<ClipboardResponse>,
    mut notification_responses: EventReader<NotificationResponse>,
    mut http_responses: EventReader<HttpResponseReceived>,
    mut search_responses: EventReader<SearchCompleted>,
    mut permission_responses: EventReader<PermissionResponse>,

    mut pending_operations: ResMut<PendingOperations>,
    mut plugin_message_writer: EventWriter<PluginMessageEvent>,
) {
    // Handle clipboard responses
    for response in clipboard_responses.read() {
        if let Some((plugin_id, _)) = pending_operations.remove_operation(&response.operation_id) {
            let ecs_response = match &response.result {
                Ok(data) => match &response.operation_type {
                    ClipboardOperationType::Read { format } => EcsEventResponse::ClipboardData {
                        data: data.clone(),
                        format: format.clone(),
                    },
                    ClipboardOperationType::Write { .. } => EcsEventResponse::ClipboardWriteSuccess,
                    ClipboardOperationType::Clear => EcsEventResponse::ClipboardClearSuccess,
                    ClipboardOperationType::CheckFormat { format } => {
                        EcsEventResponse::ClipboardFormatAvailable {
                            available: !data.is_empty(),
                        }
                    },
                },
                Err(e) => EcsEventResponse::ClipboardError {
                    error: e.to_string(),
                },
            };

            if let Ok(message) = translate_ecs_response_to_message(
                ecs_response,
                OperationId(response.operation_id),
                plugin_id,
            ) {
                plugin_message_writer.write(message);
            }
        }
    }

    // Handle notification responses
    for response in notification_responses.read() {
        if let Some((plugin_id, _)) = pending_operations.remove_operation(&response.operation_id) {
            let ecs_response = match &response.result {
                Ok(notification_id) => EcsEventResponse::NotificationShown {
                    id: notification_id.clone(),
                },
                Err(e) => EcsEventResponse::NotificationError {
                    error: e.to_string(),
                },
            };

            if let Ok(message) = translate_ecs_response_to_message(
                ecs_response,
                OperationId(response.operation_id),
                plugin_id,
            ) {
                plugin_message_writer.write(message);
            }
        }
    }

    // Handle HTTP responses
    for response in http_responses.read() {
        if let Some((plugin_id, _)) = pending_operations.remove_operation(&response.operation_id) {
            let ecs_response = match &response.result {
                Ok(http_response) => EcsEventResponse::HttpSuccess {
                    status: http_response.status,
                    body: http_response.body.clone(),
                    headers: http_response.headers.clone(),
                },
                Err(e) => EcsEventResponse::HttpError {
                    error: e.to_string(),
                    status: None,
                },
            };

            if let Ok(message) = translate_ecs_response_to_message(
                ecs_response,
                OperationId(response.operation_id),
                plugin_id,
            ) {
                plugin_message_writer.write(message);
            }
        }
    }

    // Handle search responses
    for response in search_responses.read() {
        if let Some((plugin_id, _)) = pending_operations.remove_operation(&response.operation_id) {
            let ecs_response = match &response.results {
                Ok(search_results) => {
                    let results: Vec<SearchResult> = search_results
                        .iter()
                        .map(|r| SearchResult {
                            title: r.title.clone(),
                            description: r.description.clone(),
                            action: r.action.clone(),
                            icon: r.icon.clone(),
                            score: r.score,
                            plugin_id: r.plugin_id.clone(),
                        })
                        .collect();

                    EcsEventResponse::SearchResults {
                        results,
                        search_id: response.search_id.clone(),
                    }
                },
                Err(e) => EcsEventResponse::SearchError {
                    error: e.to_string(),
                    search_id: response.search_id.clone(),
                },
            };

            if let Ok(message) = translate_ecs_response_to_message(
                ecs_response,
                OperationId(response.operation_id),
                plugin_id,
            ) {
                plugin_message_writer.write(message);
            }
        }
    }

    // Handle permission responses
    for response in permission_responses.read() {
        if let Some((plugin_id, _)) = pending_operations.remove_operation(&response.operation_id) {
            let ecs_response = match &response.result {
                Ok(permission_status) => EcsEventResponse::PermissionStatus {
                    granted: permission_status.granted,
                    status: permission_status.status.clone(),
                },
                Err(e) => EcsEventResponse::PermissionError {
                    error: e.to_string(),
                },
            };

            if let Ok(message) = translate_ecs_response_to_message(
                ecs_response,
                OperationId(response.operation_id),
                plugin_id,
            ) {
                plugin_message_writer.write(message);
            }
        }
    }
}
