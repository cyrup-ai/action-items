use bevy::prelude::*;

// Service request events - plugins send these to request services

/// Request to read from clipboard
#[derive(Event, Debug, Clone)]
pub struct ClipboardReadRequest {
    pub plugin_id: String,
    pub request_id: String,
}

/// Request to write to clipboard
#[derive(Event, Debug, Clone)]
pub struct ClipboardWriteRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub text: String,
}

/// Request to show a notification
#[derive(Event, Debug, Clone)]
pub struct NotificationRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

/// Request to read from storage
#[derive(Event, Debug, Clone)]
pub struct StorageReadRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub key: String,
}

/// Request to write to storage
#[derive(Event, Debug, Clone)]
pub struct StorageWriteRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub key: String,
    pub value: String,
}
