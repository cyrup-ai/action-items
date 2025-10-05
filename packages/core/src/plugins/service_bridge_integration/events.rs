//! Events for Service Bridge Integration
//!
//! Event types and enums for bidirectional plugin communication through ECS services

use bevy::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Response event from clipboard service operations
#[derive(Event, Debug, Clone)]
pub struct ClipboardResponseEvent {
    /// Unique operation ID for correlation
    pub operation_id: Uuid,
    /// Plugin that requested the operation
    pub plugin_id: String,
    /// Result of the clipboard operation
    pub result: Result<ClipboardResponseData, String>,
    /// Timestamp when response was generated
    pub timestamp: SystemTime,
}

/// Data returned from clipboard operations
#[derive(Debug, Clone)]
pub enum ClipboardResponseData {
    /// Text data retrieved from clipboard
    Text(String),
    /// Binary data retrieved from clipboard
    Binary(Vec<u8>),
    /// Operation completed successfully (for set/clear operations)
    Success,
}

/// Event indicating a notification was sent
#[derive(Event, Debug, Clone)]
pub struct NotificationSent {
    /// Unique notification identifier
    pub notification_id: String,
    /// Plugin that requested the notification
    pub plugin_id: String,
    /// Delivery status
    pub delivery_status: NotificationDeliveryStatus,
    /// Optional error message if delivery failed
    pub error_message: Option<String>,
    /// Timestamp when notification was sent
    pub timestamp: SystemTime,
}

/// Status of notification delivery
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationDeliveryStatus {
    /// Notification delivered successfully
    Delivered,
    /// Notification failed to deliver
    Failed,
    /// Notification delivery pending
    Pending,
    /// Notification was dismissed by user
    Dismissed,
}

/// Generic plugin response event for correlation processing
#[derive(Event, Debug, Clone)]
pub struct PluginResponseEvent {
    /// Unique operation ID for correlation
    pub operation_id: Uuid,
    /// Plugin that originated the request
    pub plugin_id: String,
    /// Type of response (matches original request type)
    pub response_type: String,
    /// Response payload data
    pub payload: Value,
    /// Success status of the operation
    pub success: bool,
    /// Optional error message
    pub error: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Timestamp when response was generated
    pub timestamp: SystemTime,
}

/// Status of async plugin tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with error
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    TimedOut,
}

impl TaskStatus {
    /// Check if task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::TimedOut)
    }

    /// Check if task is still active
    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::Pending | TaskStatus::Running)
    }
}

/// Status of operation correlation tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    /// Operation request sent, waiting for response
    WaitingForResponse,
    /// Response received, processing
    ProcessingResponse,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
    /// Operation timed out
    TimedOut,
    /// Operation was cancelled
    Cancelled,
    /// Orphaned operation (no matching response)
    Orphaned,
}

impl OperationStatus {
    /// Check if operation is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self,
            OperationStatus::Completed |
            OperationStatus::Failed |
            OperationStatus::TimedOut |
            OperationStatus::Cancelled |
            OperationStatus::Orphaned
        )
    }

    /// Check if operation is still active
    pub fn is_active(&self) -> bool {
        matches!(self, OperationStatus::WaitingForResponse | OperationStatus::ProcessingResponse)
    }
}