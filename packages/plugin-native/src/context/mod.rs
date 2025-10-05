//! Plugin context module
//!
//! Provides plugin context and service abstractions for native plugins,
//! using modern event-driven architecture for zero-allocation performance.

pub mod commands;
pub mod core;
pub mod data_types;
pub mod requests;
pub mod responses;
pub mod services;

pub use core::{CommandResult, PluginContext};

pub use commands::{
    ClipboardAction, ClipboardCommand, HttpCommand, NotificationCommand, NotificationUrgency,
    StorageAction, StorageCommand,
};
pub use data_types::{HttpMethod, HttpRequest, HttpResponseData};
pub use requests::{
    ClipboardReadRequest, ClipboardWriteRequest, NotificationRequest, StorageReadRequest,
    StorageWriteRequest,
};
pub use responses::{
    ClipboardReadResponse, ClipboardWriteResponse, HttpResponse, NotificationResponse,
    StorageReadResponse, StorageWriteResponse,
};
pub use services::{
    CacheService, ClipboardAccess, HttpClient, NotificationService, StorageService,
};
