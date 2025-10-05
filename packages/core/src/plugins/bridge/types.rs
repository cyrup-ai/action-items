//! Service bridge types for plugin communication
//!
//! This module defines the request/response types that bridge the gap between
//! the comprehensive ServiceBridge and the plugin systems, completing the
//! unified service bridge architecture.

use std::collections::HashMap;

use action_items_native::context::{
    ClipboardReadRequest, ClipboardReadResponse, ClipboardWriteRequest, ClipboardWriteResponse,
    HttpRequest, HttpResponseData, NotificationRequest, NotificationResponse, StorageReadRequest,
    StorageReadResponse, StorageWriteRequest, StorageWriteResponse,
};

/// Service request types for inter-plugin communication
#[derive(Debug, Clone)]
pub enum ServiceRequest {
    /// Request to read clipboard content
    ClipboardRead(ClipboardReadRequest),
    /// Request to write to clipboard
    ClipboardWrite(ClipboardWriteRequest),
    /// Request to show a notification
    Notification(NotificationRequest),
    /// HTTP request
    Http(HttpRequest),
    /// Request to read from storage
    StorageRead(StorageReadRequest),
    /// Request to write to storage
    StorageWrite(StorageWriteRequest),
    /// WASM callback request
    WasmCallback {
        plugin_id: String,
        function_name: String,
        data: Vec<u8>,
    },
}

/// Service response types for inter-plugin communication
#[derive(Debug)]
pub enum ServiceResponse {
    /// Response to clipboard read request
    ClipboardRead(ClipboardReadResponse),
    /// Response to clipboard write request
    ClipboardWrite(ClipboardWriteResponse),
    /// Response to notification request
    Notification(NotificationResponse),
    /// HTTP response
    Http(HttpResponseData),
    /// Response to storage read request
    StorageRead(StorageReadResponse),
    /// Response to storage write request
    StorageWrite(StorageWriteResponse),
    /// WASM callback response
    WasmCallback(Result<Vec<u8>, String>),
}

/// HTTP response wrapper for compatibility
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl From<HttpResponseData> for HttpResponse {
    fn from(data: HttpResponseData) -> Self {
        Self {
            status: data.status,
            headers: data.headers,
            body: data.body,
        }
    }
}

/// Bridge configuration for service bridge systems
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Enable request/response logging
    pub enable_logging: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            request_timeout_ms: 5000,
            enable_logging: true,
        }
    }
}

/// Bridge statistics for monitoring
#[derive(Debug, Clone)]
pub struct BridgeStats {
    /// Total requests sent
    pub requests_sent: u64,
    /// Total requests processed
    pub requests_processed: u64,
    /// Total requests failed
    pub requests_failed: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

impl BridgeStats {
    pub fn new() -> Self {
        Self {
            requests_sent: 0,
            requests_processed: 0,
            requests_failed: 0,
            avg_response_time_ms: 0.0,
        }
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.requests_processed == 0 {
            return 1.0;
        }
        let successful = self.requests_processed - self.requests_failed;
        successful as f64 / self.requests_processed as f64
    }

    /// Calculate failure rate (0.0 to 1.0)
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}

impl Default for BridgeStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge channels for communication
#[derive(Debug)]
pub struct BridgeChannels {
    /// Request sender channel
    pub request_sender: crossbeam_channel::Sender<ServiceRequest>,
    /// Request receiver channel
    pub request_receiver: crossbeam_channel::Receiver<ServiceRequest>,
    /// Response sender channel
    pub response_sender: crossbeam_channel::Sender<ServiceResponse>,
    /// Response receiver channel
    pub response_receiver: crossbeam_channel::Receiver<ServiceResponse>,
}

impl BridgeChannels {
    /// Create new bridge channels with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (request_sender, request_receiver) = crossbeam_channel::bounded(capacity);
        let (response_sender, response_receiver) = crossbeam_channel::bounded(capacity);

        Self {
            request_sender,
            request_receiver,
            response_sender,
            response_receiver,
        }
    }

    /// Create unbounded bridge channels
    pub fn unbounded() -> Self {
        let (request_sender, request_receiver) = crossbeam_channel::unbounded();
        let (response_sender, response_receiver) = crossbeam_channel::unbounded();

        Self {
            request_sender,
            request_receiver,
            response_sender,
            response_receiver,
        }
    }
}
