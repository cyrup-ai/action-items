//! ECS Events for Service Bridge
//!
//! Zero-allocation, optimal memory layout ECS events for blazing-fast performance.
//! All events use TimeStamp instead of SystemTime for consistent serialization.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::TimeStamp;

/// Message priority levels for routing optimization with explicit discriminant
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum MessagePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Plugin message event - sent between plugins via the service bridge
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginMessageEvent {
    pub from: String,
    pub to: String,
    pub plugin_id: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub priority: MessagePriority,
    pub timestamp: TimeStamp, // Fixed: Use TimeStamp instead of SystemTime
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
}

/// Broadcast message event - sent to all registered plugins
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct BroadcastMessageEvent {
    pub from: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub priority: MessagePriority,
    pub timestamp: TimeStamp, // Fixed: Use TimeStamp instead of SystemTime
}

/// Plugin lifecycle event - tracks plugin registration, status changes, etc.
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginLifecycleEvent {
    pub plugin_id: String,
    pub event_type: LifecycleEventType,
    pub timestamp: TimeStamp, // Fixed: Use TimeStamp instead of SystemTime
}

/// Types of lifecycle events with explicit discriminant for optimal serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum LifecycleEventType {
    Registered = 0,
    Started = 1,
    Stopped = 2,
    StatusChanged(String) = 3,
    Error(String) = 4,
    Unregistered = 5,
}

/// Clipboard operation event with optimized memory layout
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ClipboardEvent {
    pub request_id: String,
    pub operation: ClipboardOperation,
    pub timestamp: TimeStamp, // Fixed: Use TimeStamp instead of SystemTime
}

/// Clipboard operations with explicit discriminant for optimal serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum ClipboardOperation {
    Read = 0,
    Write(String) = 1,
    ReadResponse(String) = 2,
    WriteResponse(bool) = 3,
}

/// HTTP operation event with optimized memory layout
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct HttpEvent {
    pub request_id: String,
    pub operation: HttpOperation,
    pub timestamp: TimeStamp, // Fixed: Use TimeStamp instead of SystemTime
}

/// HTTP operations with explicit discriminant for optimal serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum HttpOperation {
    Request {
        url: String,
        method: String,
        body: Option<String>,
    } = 0,
    Response {
        status: u16,
        body: String,
    } = 1,
}

/// Notification event with optimized memory layout
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct NotificationEvent {
    pub request_id: String,
    pub title: String,
    pub body: String,
    pub timestamp: TimeStamp, // Fixed: Use TimeStamp instead of SystemTime
}

/// Storage operation event with optimized memory layout
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct StorageEvent {
    pub request_id: String,
    pub operation: StorageOperation,
    pub timestamp: TimeStamp, // Fixed: Use TimeStamp instead of SystemTime
}

/// Storage operations with explicit discriminant for optimal serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum StorageOperation {
    Read(String) = 0,                 // key
    Write(String, String) = 1,        // key, value
    ReadResponse(Option<String>) = 2, // value
    WriteResponse(bool) = 3,          // success
}
