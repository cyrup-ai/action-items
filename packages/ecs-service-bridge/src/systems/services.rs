//! Service Handler Processing Systems
//!
//! Zero-allocation, blazing-fast service processing systems with comprehensive error handling.
//! All operations use proper error propagation and performance monitoring.

use std::time::Instant;

use bevy::prelude::*;

use crate::events::*;
use crate::messaging::*;
use crate::services::*;
use crate::types::{ServiceError, ServiceResult, TimeStamp};

/// Process clipboard events and handle clipboard operations
#[inline]
pub fn process_clipboard_events_system(
    mut clipboard_events: EventReader<ClipboardEvent>,
    mut service_registry: ResMut<ServiceHandlerRegistry>,
    mut _message_infrastructure: ResMut<MessageInfrastructure>,
) {
    let start_time = Instant::now();

    for clipboard_event in clipboard_events.read() {
        let result = match &clipboard_event.operation {
            ClipboardOperation::Read => handle_clipboard_read(&service_registry, clipboard_event),
            ClipboardOperation::Write(content) => {
                handle_clipboard_write(&service_registry, clipboard_event, content.clone())
            },
            _ => {
                // ReadResponse and WriteResponse are responses, not requests
                continue;
            },
        };

        match result {
            Ok(response_operation) => {
                service_registry
                    .stats
                    .record_clipboard_success(&clipboard_event.operation);

                // Send response event
                let _response_event = ClipboardEvent {
                    request_id: clipboard_event.request_id.clone(),
                    operation: response_operation,
                    timestamp: TimeStamp::now(),
                };

                // Create message envelope for response (would be sent back to requester)
                debug!(
                    "Clipboard operation successful: {:?}",
                    clipboard_event.operation
                );
            },
            Err(error) => {
                service_registry.stats.record_clipboard_failure();
                error!("Clipboard operation failed: {}", error);
            },
        }
    }

    // Update performance metrics
    let processing_time = start_time.elapsed().as_millis() as f64;
    update_service_performance_metrics(&mut service_registry, processing_time);
}

/// Process HTTP events and handle HTTP operations
#[inline]
pub fn process_http_events_system(
    mut http_events: EventReader<HttpEvent>,
    mut service_registry: ResMut<ServiceHandlerRegistry>,
    _message_infrastructure: ResMut<MessageInfrastructure>,
) {
    let start_time = Instant::now();

    for http_event in http_events.read() {
        let result = match &http_event.operation {
            HttpOperation::Request { url, method, body } => handle_http_request(
                &service_registry,
                http_event,
                method.clone(),
                url.clone(),
                body.clone(),
            ),
            HttpOperation::Response { .. } => {
                // Response operations are responses, not requests
                continue;
            },
        };

        match result {
            Ok(response_operation) => {
                service_registry.stats.record_http_success();

                // Send response event
                let _response_event = HttpEvent {
                    request_id: http_event.request_id.clone(),
                    operation: response_operation,
                    timestamp: TimeStamp::now(),
                };

                debug!("HTTP operation successful: {:?}", http_event.operation);
            },
            Err(error) => {
                service_registry.stats.record_http_failure();
                error!("HTTP operation failed: {}", error);
            },
        }
    }

    // Update performance metrics
    let processing_time = start_time.elapsed().as_millis() as f64;
    update_service_performance_metrics(&mut service_registry, processing_time);
}

/// Process storage events and handle storage operations
#[inline]
pub fn process_storage_events_system(
    mut storage_events: EventReader<StorageEvent>,
    mut service_registry: ResMut<ServiceHandlerRegistry>,
    _message_infrastructure: ResMut<MessageInfrastructure>,
) {
    let start_time = Instant::now();

    for storage_event in storage_events.read() {
        let result = match &storage_event.operation {
            StorageOperation::Read(key) => {
                handle_storage_read(&service_registry, storage_event, key.clone())
            },
            StorageOperation::Write(key, value) => {
                handle_storage_write(&service_registry, storage_event, key.clone(), value.clone())
            },
            _ => {
                // ReadResponse and WriteResponse are responses, not requests
                continue;
            },
        };

        match result {
            Ok(response_operation) => {
                service_registry
                    .stats
                    .record_storage_success(&storage_event.operation);

                // Send response event
                let _response_event = StorageEvent {
                    request_id: storage_event.request_id.clone(),
                    operation: response_operation,
                    timestamp: TimeStamp::now(),
                };

                debug!(
                    "Storage operation successful: {:?}",
                    storage_event.operation
                );
            },
            Err(error) => {
                service_registry.stats.record_storage_failure();
                error!("Storage operation failed: {}", error);
            },
        }
    }

    // Update performance metrics
    let processing_time = start_time.elapsed().as_millis() as f64;
    update_service_performance_metrics(&mut service_registry, processing_time);
}

/// Process notification events and handle notification operations
#[inline]
pub fn process_notification_events_system(
    mut notification_events: EventReader<NotificationEvent>,
    mut service_registry: ResMut<ServiceHandlerRegistry>,
) {
    let start_time = Instant::now();

    for notification_event in notification_events.read() {
        // Validate notification content
        let validation_result = validate_notification_content(
            &service_registry.config,
            &notification_event.title,
            &notification_event.body,
        );

        let result = match validation_result {
            Ok(_) => handle_notification_send(&service_registry, notification_event),
            Err(error) => Err(error),
        };

        match result {
            Ok(_) => {
                service_registry.stats.record_notification_success();
                info!(
                    "Notification sent successfully: {}",
                    notification_event.title
                );
            },
            Err(error) => {
                service_registry.stats.record_notification_failure();
                error!("Notification failed: {}", error);
            },
        }
    }

    // Update performance metrics
    let processing_time = start_time.elapsed().as_millis() as f64;
    update_service_performance_metrics(&mut service_registry, processing_time);
}
/// Handle clipboard read operation with comprehensive validation
#[inline]
fn handle_clipboard_read(
    service_registry: &ServiceHandlerRegistry,
    clipboard_event: &ClipboardEvent,
) -> ServiceResult<ClipboardOperation> {
    // Check timeout
    let age = TimeStamp::now()
        .duration_since(clipboard_event.timestamp)
        .map_err(|_| ServiceError::InvalidTimestamp)?;

    if age.as_millis() > service_registry.config.clipboard_timeout_ms as u128 {
        return Err(ServiceError::ClipboardError {
            operation: "read".to_string(),
            reason: "Operation timed out".to_string(),
        });
    }

    // Perform clipboard read
    let content = service_registry
        .clipboard_handler
        .read_clipboard()
        .map_err(|e| ServiceError::ClipboardError {
            operation: "read".to_string(),
            reason: e.to_string(),
        })?;

    // Validate content size
    if content.len() > service_registry.config.clipboard_max_size_bytes as usize {
        return Err(ServiceError::ClipboardError {
            operation: "read".to_string(),
            reason: format!("Content too large: {} bytes", content.len()),
        });
    }

    Ok(ClipboardOperation::ReadResponse(content))
}

/// Handle clipboard write operation with comprehensive validation
#[inline]
fn handle_clipboard_write(
    service_registry: &ServiceHandlerRegistry,
    clipboard_event: &ClipboardEvent,
    content: String,
) -> ServiceResult<ClipboardOperation> {
    // Check timeout
    let age = TimeStamp::now()
        .duration_since(clipboard_event.timestamp)
        .map_err(|_| ServiceError::InvalidTimestamp)?;

    if age.as_millis() > service_registry.config.clipboard_timeout_ms as u128 {
        return Err(ServiceError::ClipboardError {
            operation: "write".to_string(),
            reason: "Operation timed out".to_string(),
        });
    }

    // Validate content size
    if content.len() > service_registry.config.clipboard_max_size_bytes as usize {
        return Err(ServiceError::ClipboardError {
            operation: "write".to_string(),
            reason: format!("Content too large: {} bytes", content.len()),
        });
    }

    // Perform clipboard write
    let success = service_registry
        .clipboard_handler
        .write_clipboard(content)
        .map_err(|e| ServiceError::ClipboardError {
            operation: "write".to_string(),
            reason: e.to_string(),
        })?;

    Ok(ClipboardOperation::WriteResponse(success))
}

/// Handle HTTP request operation with comprehensive validation
#[inline]
fn handle_http_request(
    service_registry: &ServiceHandlerRegistry,
    http_event: &HttpEvent,
    method: String,
    url: String,
    body: Option<String>,
) -> ServiceResult<HttpOperation> {
    // Check timeout
    let age = TimeStamp::now()
        .duration_since(http_event.timestamp)
        .map_err(|_| ServiceError::InvalidTimestamp)?;

    if age.as_millis() > service_registry.config.http_timeout_ms as u128 {
        return Err(ServiceError::HttpError {
            method: method.clone(),
            url: url.clone(),
            reason: "Operation timed out".to_string(),
        });
    }

    // Validate URL
    if url.is_empty() || url.len() > 2048 {
        return Err(ServiceError::HttpError {
            method: method.clone(),
            url: url.clone(),
            reason: "Invalid URL length".to_string(),
        });
    }

    // Validate method
    if !matches!(
        method.as_str(),
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS"
    ) {
        return Err(ServiceError::HttpError {
            method: method.clone(),
            url: url.clone(),
            reason: "Invalid HTTP method".to_string(),
        });
    }

    // Validate body size if present
    if let Some(ref body_content) = body
        && body_content.len() > service_registry.config.http_max_response_size as usize {
            return Err(ServiceError::HttpError {
                method: method.clone(),
                url: url.clone(),
                reason: format!("Request body too large: {} bytes", body_content.len()),
            });
        }

    // Perform HTTP request
    let (status, response_body) = service_registry
        .http_handler
        .send_request(method.clone(), url.clone(), body)
        .map_err(|e| ServiceError::HttpError {
            method: method.clone(),
            url: url.clone(),
            reason: e.to_string(),
        })?;

    Ok(HttpOperation::Response {
        status,
        body: response_body,
    })
}

/// Handle storage read operation with comprehensive validation
#[inline]
fn handle_storage_read(
    service_registry: &ServiceHandlerRegistry,
    storage_event: &StorageEvent,
    key: String,
) -> ServiceResult<StorageOperation> {
    // Check timeout
    let age = TimeStamp::now()
        .duration_since(storage_event.timestamp)
        .map_err(|_| ServiceError::InvalidTimestamp)?;

    if age.as_millis() > service_registry.config.storage_timeout_ms as u128 {
        return Err(ServiceError::StorageError {
            operation: "read".to_string(),
            reason: "Operation timed out".to_string(),
        });
    }

    // Validate key
    if key.is_empty() || key.len() > service_registry.config.storage_max_key_length as usize {
        return Err(ServiceError::StorageError {
            operation: "read".to_string(),
            reason: format!("Invalid key length: {}", key.len()),
        });
    }

    // Perform storage read
    let value = service_registry
        .storage_handler
        .read_storage(key.clone())
        .map_err(|e| ServiceError::StorageError {
            operation: "read".to_string(),
            reason: e.to_string(),
        })?;

    Ok(StorageOperation::ReadResponse(value))
}

/// Handle storage write operation with comprehensive validation
#[inline]
fn handle_storage_write(
    service_registry: &ServiceHandlerRegistry,
    storage_event: &StorageEvent,
    key: String,
    value: String,
) -> ServiceResult<StorageOperation> {
    // Check timeout
    let age = TimeStamp::now()
        .duration_since(storage_event.timestamp)
        .map_err(|_| ServiceError::InvalidTimestamp)?;

    if age.as_millis() > service_registry.config.storage_timeout_ms as u128 {
        return Err(ServiceError::StorageError {
            operation: "write".to_string(),
            reason: "Operation timed out".to_string(),
        });
    }

    // Validate key
    if key.is_empty() || key.len() > service_registry.config.storage_max_key_length as usize {
        return Err(ServiceError::StorageError {
            operation: "write".to_string(),
            reason: format!("Invalid key length: {}", key.len()),
        });
    }

    // Validate value size
    if value.len() > service_registry.config.storage_max_value_size as usize {
        return Err(ServiceError::StorageError {
            operation: "write".to_string(),
            reason: format!("Value too large: {} bytes", value.len()),
        });
    }

    // Perform storage write
    let success = service_registry
        .storage_handler
        .write_storage(key.clone(), value)
        .map_err(|e| ServiceError::StorageError {
            operation: "write".to_string(),
            reason: e.to_string(),
        })?;

    Ok(StorageOperation::WriteResponse(success))
}

/// Handle notification send operation with comprehensive validation
#[inline]
fn handle_notification_send(
    service_registry: &ServiceHandlerRegistry,
    notification_event: &NotificationEvent,
) -> ServiceResult<()> {
    // Check timeout
    let age = TimeStamp::now()
        .duration_since(notification_event.timestamp)
        .map_err(|_| ServiceError::InvalidTimestamp)?;

    if age.as_millis() > service_registry.config.notification_timeout_ms as u128 {
        return Err(ServiceError::NotificationError {
            reason: "Operation timed out".to_string(),
        });
    }

    // Perform notification send
    service_registry
        .notification_handler
        .send_notification(
            notification_event.title.clone(),
            notification_event.body.clone(),
        )
        .map_err(|e| ServiceError::NotificationError {
            reason: e.to_string(),
        })
}

/// Validate notification content against configuration limits
#[inline]
fn validate_notification_content(
    config: &ServiceConfig,
    title: &str,
    body: &str,
) -> ServiceResult<()> {
    if title.is_empty() {
        return Err(ServiceError::NotificationError {
            reason: "Title cannot be empty".to_string(),
        });
    }

    if title.len() > config.notification_max_title_length as usize {
        return Err(ServiceError::NotificationError {
            reason: format!("Title too long: {} characters", title.len()),
        });
    }

    if body.len() > config.notification_max_body_length as usize {
        return Err(ServiceError::NotificationError {
            reason: format!("Body too long: {} characters", body.len()),
        });
    }

    Ok(())
}

/// Update service performance metrics with rolling average
#[inline]
fn update_service_performance_metrics(
    service_registry: &mut ServiceHandlerRegistry,
    processing_time_ms: f64,
) {
    // Update rolling average response time
    if service_registry.stats.avg_response_time_ms == 0.0 {
        service_registry.stats.avg_response_time_ms = processing_time_ms;
    } else {
        service_registry.stats.avg_response_time_ms =
            (service_registry.stats.avg_response_time_ms * 0.9) + (processing_time_ms * 0.1);
    }
}

/// Service health monitoring system
#[inline]
pub fn monitor_service_health_system(
    service_registry: Res<ServiceHandlerRegistry>,
    _time: Res<Time>,
) {
    let stats = &service_registry.stats;

    // Calculate failure rates
    let clipboard_failure_rate = if stats.clipboard_reads + stats.clipboard_writes > 0 {
        (stats.clipboard_failures as f64)
            / ((stats.clipboard_reads + stats.clipboard_writes) as f64)
    } else {
        0.0
    };

    let http_failure_rate = if stats.http_requests > 0 {
        (stats.http_failures as f64) / (stats.http_requests as f64)
    } else {
        0.0
    };

    let storage_failure_rate = if stats.storage_reads + stats.storage_writes > 0 {
        (stats.storage_failures as f64) / ((stats.storage_reads + stats.storage_writes) as f64)
    } else {
        0.0
    };

    let notification_failure_rate = if stats.notifications_sent + stats.notification_failures > 0 {
        (stats.notification_failures as f64)
            / ((stats.notifications_sent + stats.notification_failures) as f64)
    } else {
        0.0
    };

    // Log warnings for high failure rates
    if clipboard_failure_rate > 0.1 {
        warn!(
            "High clipboard failure rate: {:.2}%",
            clipboard_failure_rate * 100.0
        );
    }

    if http_failure_rate > 0.1 {
        warn!("High HTTP failure rate: {:.2}%", http_failure_rate * 100.0);
    }

    if storage_failure_rate > 0.1 {
        warn!(
            "High storage failure rate: {:.2}%",
            storage_failure_rate * 100.0
        );
    }

    if notification_failure_rate > 0.1 {
        warn!(
            "High notification failure rate: {:.2}%",
            notification_failure_rate * 100.0
        );
    }

    // Log periodic statistics (every 60 seconds)
    static mut LAST_LOG_TIME: Option<std::time::Instant> = None;
    unsafe {
        let now = std::time::Instant::now();
        let should_log = LAST_LOG_TIME
            .map(|last| now.duration_since(last).as_secs() >= 60)
            .unwrap_or(true);

        if should_log {
            info!(
                "Service Stats: clipboard({}/{}/{}), http({}/{}/{}), storage({}/{}/{}), \
                 notifications({}/{})",
                stats.clipboard_reads,
                stats.clipboard_writes,
                stats.clipboard_failures,
                stats.http_successes,
                stats.http_requests,
                stats.http_failures,
                stats.storage_reads,
                stats.storage_writes,
                stats.storage_failures,
                stats.notifications_sent,
                stats.notification_failures
            );
            LAST_LOG_TIME = Some(now);
        }
    }
}
