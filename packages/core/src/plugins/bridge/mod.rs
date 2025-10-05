//! Plugin bridge module
//!
//! Provides service bridge functionality for external thread communication
//! with Bevy systems, including request handling, WASM callbacks, and monitoring.

pub mod handlers;
pub mod services;
pub mod services_bevy;
pub mod systems;
pub mod types;

// Re-export from correct modules
pub use handlers::{
    handle_clipboard_read, handle_clipboard_write, handle_http_request, handle_notification,
    handle_storage_delete, handle_storage_read, handle_storage_write, process_service_request,
};
pub use services::{ClipboardAccess, HttpClient, NotificationService, StorageService};
pub use services_bevy::{
    ClipboardService, NotificationService as BevyNotificationService,
    StorageService as BevyStorageService,
};
pub use systems::{
    ServiceBridgePlugin as Plugin, ServiceBridgePlugin, SharedServiceBridge,
    service_bridge_cleanup_system, service_bridge_monitor_system, service_bridge_system,
    service_bridge_system as bridge_service_system, wasm_callback_system_ecs,
};
pub use types::{HttpResponse, ServiceRequest, ServiceResponse};

pub use crate::service_bridge::bridge::core::ServiceBridge; // Add ServiceBridge import
