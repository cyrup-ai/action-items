//! Service bridge handlers module
//!
//! This module provides handlers for various service requests from plugins,
//! including clipboard operations, notifications, storage, and HTTP requests.
//!
//! The module has been decomposed into logical submodules:
//! - `clipboard` - Clipboard read/write operations
//! - `notifications` - System notification handling
//! - `storage` - File-based storage operations (read/write/delete)
//! - `http` - HTTP request processing
//! - `processor` - Service request routing and processing

pub mod clipboard;
pub mod http;
pub mod notifications;
pub mod processor;
pub mod storage;

// Re-export main processing function
// Re-export individual handler functions
pub use clipboard::{handle_clipboard_read, handle_clipboard_write};
pub use http::handle_http_request;
pub use notifications::handle_notification;
pub use processor::process_service_request;
pub use storage::{handle_storage_delete, handle_storage_read, handle_storage_write};
