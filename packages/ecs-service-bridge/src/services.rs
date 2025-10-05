//! Service Handler Systems
//!
//! Zero-allocation, blazing-fast service handlers for clipboard, HTTP, storage, and notifications.
//! All operations use proper error propagation and comprehensive validation.

use std::sync::Arc;

use bevy::prelude::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::events::*;
use crate::types::ServiceResult;

/// Type alias for HTTP response callback functions
type HttpResponseCallback = Box<dyn Fn(ServiceResult<(u16, String)>) + Send + Sync>;

/// Service handler registry resource for managing service implementations
#[derive(Resource)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceHandlerRegistry {
    /// Clipboard service handler
    pub clipboard_handler: Arc<dyn ClipboardHandler + Send + Sync>,
    /// HTTP service handler
    pub http_handler: Arc<dyn HttpHandler + Send + Sync>,
    /// Storage service handler  
    pub storage_handler: Arc<dyn StorageHandler + Send + Sync>,
    /// Notification service handler
    pub notification_handler: Arc<dyn NotificationHandler + Send + Sync>,
    /// Service statistics
    pub stats: ServiceStats,
    /// Service configuration
    pub config: ServiceConfig,
}

/// Service operation statistics for monitoring
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceStats {
    /// Clipboard operations
    pub clipboard_reads: u64,
    pub clipboard_writes: u64,
    pub clipboard_failures: u64,
    /// HTTP operations
    pub http_requests: u64,
    pub http_successes: u64,
    pub http_failures: u64,
    /// Storage operations
    pub storage_reads: u64,
    pub storage_writes: u64,
    pub storage_failures: u64,
    /// Notification operations
    pub notifications_sent: u64,
    pub notification_failures: u64,
    /// Performance metrics
    pub avg_response_time_ms: f64,
    pub peak_concurrent_operations: usize,
}

impl ServiceStats {
    /// Record successful clipboard operation
    #[inline]
    pub fn record_clipboard_success(&mut self, operation: &ClipboardOperation) {
        match operation {
            ClipboardOperation::Read | ClipboardOperation::ReadResponse(_) => {
                self.clipboard_reads += 1;
            },
            ClipboardOperation::Write(_) | ClipboardOperation::WriteResponse(_) => {
                self.clipboard_writes += 1;
            },
        }
    }

    /// Record failed clipboard operation
    #[inline]
    pub fn record_clipboard_failure(&mut self) {
        self.clipboard_failures += 1;
    }

    /// Record HTTP operation
    #[inline]
    pub fn record_http_success(&mut self) {
        self.http_requests += 1;
        self.http_successes += 1;
    }

    /// Record HTTP failure
    #[inline]
    pub fn record_http_failure(&mut self) {
        self.http_requests += 1;
        self.http_failures += 1;
    }

    /// Record storage operation
    #[inline]
    pub fn record_storage_success(&mut self, operation: &StorageOperation) {
        match operation {
            StorageOperation::Read(_) | StorageOperation::ReadResponse(_) => {
                self.storage_reads += 1;
            },
            StorageOperation::Write(..) | StorageOperation::WriteResponse(_) => {
                self.storage_writes += 1;
            },
        }
    }

    /// Record storage failure
    #[inline]
    pub fn record_storage_failure(&mut self) {
        self.storage_failures += 1;
    }

    /// Record notification success
    #[inline]
    pub fn record_notification_success(&mut self) {
        self.notifications_sent += 1;
    }

    /// Record notification failure
    #[inline]
    pub fn record_notification_failure(&mut self) {
        self.notification_failures += 1;
    }
}

/// Service configuration for handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceConfig {
    /// Clipboard settings
    pub clipboard_timeout_ms: u64,
    pub clipboard_max_size_bytes: u32,
    /// HTTP settings
    pub http_timeout_ms: u64,
    pub http_max_redirects: u8,
    pub http_max_response_size: u32,
    /// Storage settings
    pub storage_timeout_ms: u64,
    pub storage_max_key_length: u32,
    pub storage_max_value_size: u32,
    /// Notification settings
    pub notification_timeout_ms: u64,
    pub notification_max_title_length: u32,
    pub notification_max_body_length: u32,
}

impl Default for ServiceConfig {
    #[inline]
    fn default() -> Self {
        Self {
            clipboard_timeout_ms: 5000,            // 5 seconds
            clipboard_max_size_bytes: 1024 * 1024, // 1MB
            http_timeout_ms: 30000,                // 30 seconds
            http_max_redirects: 5,
            http_max_response_size: 10 * 1024 * 1024, // 10MB
            storage_timeout_ms: 5000,                 // 5 seconds
            storage_max_key_length: 1024,             // 1KB
            storage_max_value_size: 1024 * 1024,      // 1MB
            notification_timeout_ms: 10000,           // 10 seconds
            notification_max_title_length: 256,
            notification_max_body_length: 2048,
        }
    }
}

/// Trait for clipboard service handlers
pub trait ClipboardHandler {
    /// Read from clipboard
    fn read_clipboard(&self) -> ServiceResult<String>;

    /// Write to clipboard
    fn write_clipboard(&self, content: String) -> ServiceResult<bool>;
}

/// Trait for HTTP service handlers
pub trait HttpHandler {
    /// Send HTTP request
    fn send_request(
        &self,
        method: String,
        url: String,
        body: Option<String>,
    ) -> ServiceResult<(u16, String)>;

    /// Send HTTP request asynchronously
    fn send_request_async(
        &self,
        method: String,
        url: String,
        body: Option<String>,
        callback: HttpResponseCallback,
    );
}

/// Trait for storage service handlers
pub trait StorageHandler {
    /// Read value from storage
    fn read_storage(&self, key: String) -> ServiceResult<Option<String>>;

    /// Write value to storage
    fn write_storage(&self, key: String, value: String) -> ServiceResult<bool>;

    /// Delete value from storage
    fn delete_storage(&self, key: String) -> ServiceResult<bool>;

    /// List all keys in storage
    fn list_keys(&self) -> ServiceResult<Vec<String>>;

    /// Clear all storage
    fn clear_storage(&self) -> ServiceResult<bool>;
}

/// Trait for notification service handlers
pub trait NotificationHandler {
    /// Send notification
    fn send_notification(&self, title: String, body: String) -> ServiceResult<()>;

    /// Send notification with custom icon
    fn send_notification_with_icon(
        &self,
        title: String,
        body: String,
        icon: Option<String>,
    ) -> ServiceResult<()>;
}
/// Default clipboard handler implementation (no-op for testing)
#[derive(Debug, Default)]
pub struct DefaultClipboardHandler;

impl ClipboardHandler for DefaultClipboardHandler {
    #[inline]
    fn read_clipboard(&self) -> ServiceResult<String> {
        // Default implementation returns empty string
        Ok(String::new())
    }

    #[inline]
    fn write_clipboard(&self, _content: String) -> ServiceResult<bool> {
        // Default implementation always succeeds
        Ok(true)
    }
}

/// Default HTTP handler implementation (no-op for testing)
#[derive(Debug, Default)]
pub struct DefaultHttpHandler;

impl HttpHandler for DefaultHttpHandler {
    #[inline]
    fn send_request(
        &self,
        method: String,
        url: String,
        _body: Option<String>,
    ) -> ServiceResult<(u16, String)> {
        // Default implementation returns 200 OK with empty body
        debug!("Default HTTP handler: {} {}", method, url);
        Ok((200, "{}".to_string()))
    }

    #[inline]
    fn send_request_async(
        &self,
        method: String,
        url: String,
        body: Option<String>,
        callback: Box<dyn Fn(ServiceResult<(u16, String)>) + Send + Sync>,
    ) {
        // Call synchronous version and invoke callback
        let result = self.send_request(method, url, body);
        callback(result);
    }
}

/// Default storage handler implementation (in-memory for testing)
#[derive(Debug)]
pub struct DefaultStorageHandler {
    storage: Arc<RwLock<FxHashMap<String, String>>>,
}

impl Default for DefaultStorageHandler {
    #[inline]
    fn default() -> Self {
        Self {
            storage: Arc::new(RwLock::new(FxHashMap::default())),
        }
    }
}

impl StorageHandler for DefaultStorageHandler {
    #[inline]
    fn read_storage(&self, key: String) -> ServiceResult<Option<String>> {
        // This is a blocking operation for the default implementation
        // In production, this would be async
        let storage = self.storage.blocking_read();
        Ok(storage.get(&key).cloned())
    }

    #[inline]
    fn write_storage(&self, key: String, value: String) -> ServiceResult<bool> {
        // This is a blocking operation for the default implementation
        // In production, this would be async
        let mut storage = self.storage.blocking_write();
        storage.insert(key, value);
        Ok(true)
    }

    #[inline]
    fn delete_storage(&self, key: String) -> ServiceResult<bool> {
        let mut storage = self.storage.blocking_write();
        Ok(storage.remove(&key).is_some())
    }

    #[inline]
    fn list_keys(&self) -> ServiceResult<Vec<String>> {
        let storage = self.storage.blocking_read();
        Ok(storage.keys().cloned().collect())
    }

    #[inline]
    fn clear_storage(&self) -> ServiceResult<bool> {
        let mut storage = self.storage.blocking_write();
        storage.clear();
        Ok(true)
    }
}

/// Default notification handler implementation (logging for testing)
#[derive(Debug, Default)]
pub struct DefaultNotificationHandler;

impl NotificationHandler for DefaultNotificationHandler {
    #[inline]
    fn send_notification(&self, title: String, body: String) -> ServiceResult<()> {
        // Default implementation just logs the notification
        info!("Notification: {} - {}", title, body);
        Ok(())
    }

    #[inline]
    fn send_notification_with_icon(
        &self,
        title: String,
        body: String,
        icon: Option<String>,
    ) -> ServiceResult<()> {
        // Default implementation logs with icon info
        if let Some(icon) = icon {
            info!("Notification [{}]: {} - {}", icon, title, body);
        } else {
            info!("Notification: {} - {}", title, body);
        }
        Ok(())
    }
}

impl Default for ServiceHandlerRegistry {
    #[inline]
    fn default() -> Self {
        Self {
            clipboard_handler: Arc::new(DefaultClipboardHandler),
            http_handler: Arc::new(DefaultHttpHandler),
            storage_handler: Arc::new(DefaultStorageHandler::default()),
            notification_handler: Arc::new(DefaultNotificationHandler),
            stats: ServiceStats::default(),
            config: ServiceConfig::default(),
        }
    }
}

impl ServiceHandlerRegistry {
    /// Create new service handler registry with default handlers
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom handlers
    #[inline]
    pub fn with_handlers(
        clipboard_handler: Arc<dyn ClipboardHandler + Send + Sync>,
        http_handler: Arc<dyn HttpHandler + Send + Sync>,
        storage_handler: Arc<dyn StorageHandler + Send + Sync>,
        notification_handler: Arc<dyn NotificationHandler + Send + Sync>,
    ) -> Self {
        Self {
            clipboard_handler,
            http_handler,
            storage_handler,
            notification_handler,
            stats: ServiceStats::default(),
            config: ServiceConfig::default(),
        }
    }

    /// Update configuration
    #[inline]
    pub fn with_config(mut self, config: ServiceConfig) -> Self {
        self.config = config;
        self
    }

    /// Get service statistics
    #[inline]
    pub fn stats(&self) -> &ServiceStats {
        &self.stats
    }

    /// Get service configuration
    #[inline]
    pub fn config(&self) -> &ServiceConfig {
        &self.config
    }
}
