pub mod builder;
pub mod context;
pub mod error;
pub mod events;
pub mod ffi;
pub mod native;
pub mod traits;
pub mod views;

// Import canonical types from common - ONE UNIFIED SYSTEM
pub use action_items_common::plugin_interface::{
    ActionDefinition, ActionItem, CommandDefinition, ConfigurationField, PluginCapabilities,
    PluginCategory, PluginManifest, PluginPermissions, PreferenceField,
};
// Re-export the macro
pub use action_items_native_macros::plugin;
// Re-export native implementation details only
pub use builder::PluginBuilder;
pub use context::{
    CacheService,
    ClipboardAccess,
    // Modern event types
    ClipboardReadRequest,
    ClipboardReadResponse,
    ClipboardWriteRequest,
    ClipboardWriteResponse,
    CommandResult as ContextCommandResult,
    HttpClient,
    HttpMethod,
    HttpRequest,
    HttpResponse,
    HttpResponseData,
    NotificationRequest,
    NotificationResponse,
    NotificationService,
    PluginContext,
    StorageReadRequest,
    StorageReadResponse,
    StorageService,
    StorageWriteRequest,
    StorageWriteResponse,
};
pub use error::{Error, Result};
pub use events::*;
pub use ffi::*;
pub use native::*;
pub use traits::*;
pub use views::{CommandResult as ViewCommandResult, DetailView, FormField, FormView};

// Re-export command types from core interface
pub use crate::context::{
    ClipboardAction, ClipboardCommand, HttpCommand, NotificationCommand, NotificationUrgency,
    StorageAction, StorageCommand,
};
