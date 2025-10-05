use bevy::prelude::*;

use super::data_types::HttpResponseData;

// Service response events - systems send these when operations complete

/// Response from clipboard read
#[derive(Event, Clone, Debug)]
pub struct ClipboardReadResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<String, String>,
}

/// Response from clipboard write
#[derive(Event, Clone, Debug)]
pub struct ClipboardWriteResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<(), String>,
}

/// Response from notification
#[derive(Event, Clone, Debug)]
pub struct NotificationResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub notification_id: Result<String, String>,
}

/// Response from storage read
#[derive(Event, Clone, Debug)]
pub struct StorageReadResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<String, String>,
}

/// Response from storage write
#[derive(Event, Clone, Debug)]
pub struct StorageWriteResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<(), String>,
}

/// Response from HTTP request
#[derive(Event, Clone, Debug)]
pub struct HttpResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<HttpResponseData, String>,
}
