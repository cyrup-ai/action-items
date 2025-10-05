//! Event types for user settings requests and responses
//!
//! # Architecture
//!
//! Events follow a request/response pattern:
//! 1. System sends a `*Requested` event with operation_id and requester entity
//! 2. Async task processes the request
//! 3. Completion event (`*Completed`) sent back to requester with result
//!
//! # Event Flow
//!
//! ```text
//! System A                    Settings Service                 System A
//!    |                               |                             |
//!    |--SettingsReadRequested------->|                             |
//!    |   (operation_id: uuid)        |                             |
//!    |                               |--[async task]-->            |
//!    |                               |                             |
//!    |<--SettingsReadCompleted-------|                             |
//!    |   (operation_id: uuid)        |                             |
//!    |   (result: Ok/Err)            |                             |
//! ```
//!
//! # Change Notifications
//!
//! All mutations (write, update, delete) emit `SettingChanged` events for:
//! - Audit trail recording (see [`systems::write_audit_trail`])
//! - Live update notifications to interested systems
//! - Complete change history with old_value and new_value

use bevy::prelude::*;
use surrealdb::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::SettingsError;

// ============================================================================
// Request Events
// ============================================================================

/// Request to read a setting
#[derive(Event, Debug, Clone)]
pub struct SettingsReadRequested {
    pub operation_id: Uuid,
    pub table: String,           // "user_preferences", "hotkey_settings", etc.
    pub key: String,              // Record ID
    pub requester: Entity,
}

/// Request to write a setting (CREATE)
#[derive(Event, Debug, Clone)]
pub struct SettingsWriteRequested {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub value: Value,             // Full record as JSON
    pub requester: Entity,
}

/// Request to update a setting (UPDATE specific fields)
#[derive(Event, Debug, Clone)]
pub struct SettingsUpdateRequested {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub fields: HashMap<String, Value>,  // Partial update
    pub requester: Entity,
}

/// Request to delete a setting
#[derive(Event, Debug, Clone)]
pub struct SettingsDeleteRequested {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub requester: Entity,
}

/// Request to query settings with SurrealQL
#[derive(Event, Debug, Clone)]
pub struct SettingsQueryRequested {
    pub operation_id: Uuid,
    pub query: String,
    pub params: Option<HashMap<String, Value>>,
    pub requester: Entity,
}

// ============================================================================
// Response Events
// ============================================================================

/// Response to a read request
#[derive(Event, Debug, Clone)]
pub struct SettingsReadCompleted {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub result: Result<Option<Value>, SettingsError>,
    pub requester: Entity,
}

/// Response to a write request
#[derive(Event, Debug, Clone)]
pub struct SettingsWriteCompleted {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub result: Result<(), SettingsError>,
    pub requester: Entity,
}

/// Response to an update request
#[derive(Event, Debug, Clone)]
pub struct SettingsUpdateCompleted {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub result: Result<(), SettingsError>,
    pub requester: Entity,
}

/// Response to a delete request
#[derive(Event, Debug, Clone)]
pub struct SettingsDeleteCompleted {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub result: Result<bool, SettingsError>,  // true if existed
    pub requester: Entity,
}

/// Response to a query request
#[derive(Event, Debug, Clone)]
pub struct SettingsQueryCompleted {
    pub operation_id: Uuid,
    pub result: Result<Vec<Value>, SettingsError>,
    pub requester: Entity,
}

// ============================================================================
// Notification Events
// ============================================================================

/// Broadcast when any setting changes
#[derive(Event, Debug, Clone)]
pub struct SettingChanged {
    pub table: String,
    pub key: String,
    pub old_value: Option<Value>,
    pub new_value: Value,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}
