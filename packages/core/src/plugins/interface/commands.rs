//! Command types for plugin interface
//!
//! These command types provide a unified interface for plugin operations,
//! wrapping the underlying request/response types.

use std::collections::HashMap;

use action_items_native::context::HttpMethod;
use serde::{Deserialize, Serialize};

/// Command for clipboard operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardCommand {
    pub action: ClipboardAction,
    pub content: Option<String>,
}

/// Clipboard action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardAction {
    Read,
    Write,
}

/// Command for HTTP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCommand {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// Command for notification operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCommand {
    pub title: String,
    pub message: String,
    pub urgency: NotificationUrgency,
    pub icon: Option<String>,
}

/// Notification urgency levels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Command for storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCommand {
    pub action: StorageAction,
    pub key: String,
    pub value: Option<serde_json::Value>,
}

/// Storage action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageAction {
    Read,
    Write,
    Delete,
}
