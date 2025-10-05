//! Service request and response events for plugin operations

use std::collections::HashMap;

use bevy::prelude::*;
use serde;

// Service request events - plugins send these to request services

/// Request to read from clipboard
#[derive(Event)]
pub struct ClipboardReadRequest {
    pub plugin_id: String,
    pub request_id: String,
}

/// Request to write to clipboard
#[derive(Event)]
pub struct ClipboardWriteRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub text: String,
}

/// Request to show a notification
#[derive(Event)]
pub struct NotificationRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

/// Request to read from storage
#[derive(Event)]
pub struct StorageReadRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub key: String,
}

/// Request to write to storage
#[derive(Event)]
pub struct StorageWriteRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub key: String,
    pub value: String,
}

/// Request to perform HTTP operation
#[derive(serde::Deserialize, serde::Serialize, Debug, Event, Clone)]
pub struct HttpRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

// Service response events - systems send these when operations complete

/// Response from clipboard read
#[derive(Event)]
pub struct ClipboardReadResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<String, String>,
}

/// Response from clipboard write
#[derive(Event)]
pub struct ClipboardWriteResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<(), String>,
}

/// Response from notification
#[derive(Event)]
pub struct NotificationResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub notification_id: Result<String, String>,
}

/// Response from storage read
#[derive(Event)]
pub struct StorageReadResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<String, String>,
}

/// Response from storage write
#[derive(Event)]
pub struct StorageWriteResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<(), String>,
}

/// Response from HTTP request
#[derive(Event)]
pub struct HttpResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<HttpResponseData, String>,
}

// Data types

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct HttpResponseData {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}
